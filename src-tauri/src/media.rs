use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::models::MediaProbe;

pub fn command_exists(name: &str) -> bool {
    Command::new(name).arg("-version").output().is_ok()
}

/// Returns true if this FFmpeg build includes the drawtext filter (requires libfreetype).
pub fn drawtext_supported() -> bool {
    let out = Command::new("ffmpeg")
        .args(["-filters", "-v", "quiet"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("drawtext"),
        Err(_) => false,
    }
}

pub fn probe_media(path: &str) -> Result<MediaProbe> {
    if !command_exists("ffprobe") {
        return Err(anyhow!("ffprobe is not installed or not available on PATH"));
    }

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .context("running ffprobe")?;

    if !output.status.success() {
        return Err(anyhow!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let json: Value = serde_json::from_slice(&output.stdout).context("parsing ffprobe JSON")?;
    let streams = json
        .get("streams")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let video = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("video"));
    let audio = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("audio"));

    let duration_sec = json
        .get("format")
        .and_then(|format| format.get("duration"))
        .and_then(Value::as_str)
        .and_then(|duration| duration.parse::<f64>().ok());

    Ok(MediaProbe {
        duration_sec,
        has_video: video.is_some(),
        width: video
            .and_then(|stream| stream.get("width"))
            .and_then(Value::as_i64),
        height: video
            .and_then(|stream| stream.get("height"))
            .and_then(Value::as_i64),
        video_codec: video
            .and_then(|stream| stream.get("codec_name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        audio_codec: audio
            .and_then(|stream| stream.get("codec_name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

pub fn extract_audio(source_path: &str, project_dir: &Path) -> Result<PathBuf> {
    extract_audio_with_progress(source_path, project_dir, 0.0, None)
}

pub fn extract_audio_with_progress(
    source_path: &str,
    project_dir: &Path,
    duration_sec: f64,
    on_progress: Option<Box<dyn Fn(f64) + Send>>,
) -> Result<PathBuf> {
    if !command_exists("ffmpeg") {
        return Err(anyhow!("ffmpeg is not installed or not available on PATH"));
    }

    std::fs::create_dir_all(project_dir)?;
    let output_path = project_dir.join("transcription_audio.wav");

    let mut child = Command::new("ffmpeg")
        .args(["-y", "-i", source_path, "-vn", "-ac", "1", "-ar", "16000"])
        .arg(&output_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("running ffmpeg audio extraction")?;

    if let (Some(stderr), Some(ref on_prog)) = (child.stderr.take(), on_progress) {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stderr);
        for line in reader.split(b'\r') {
            if let Ok(line) = String::from_utf8(line.unwrap_or_default()) {
                if let Some(time_idx) = line.find("time=") {
                    let time_str = &line[time_idx + 5..];
                    if let Some(space_idx) = time_str.find(' ') {
                        let time_str = &time_str[..space_idx];
                        let parts: Vec<&str> = time_str.split(':').collect();
                        if parts.len() == 3 {
                            let h = parts[0].parse::<f64>().unwrap_or(0.0);
                            let m = parts[1].parse::<f64>().unwrap_or(0.0);
                            let s = parts[2].parse::<f64>().unwrap_or(0.0);
                            let sec = h * 3600.0 + m * 60.0 + s;
                            if duration_sec > 0.0 {
                                let mut percent = (sec / duration_sec) * 100.0;
                                if percent > 100.0 {
                                    percent = 100.0;
                                }
                                on_prog(percent);
                            }
                        }
                    }
                }
            }
        }
    }

    let output = child.wait_with_output().context("waiting for ffmpeg audio extraction")?;

    if !output.status.success() {
        return Err(anyhow!(
            "ffmpeg audio extraction failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(output_path)
}


pub fn render_flat_clip(
    source_path: &str,
    segments: &[crate::models::SegmentRange],
    output_path: &Path,
    drawtext_filters: Option<&str>,
) -> Result<PathBuf> {
    if !command_exists("ffmpeg") {
        return Err(anyhow!("ffmpeg is not installed or not available on PATH"));
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let probe = probe_media(source_path).ok();
    let has_video = probe.map(|p| p.has_video).unwrap_or(false);

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-i", source_path]);

    if segments.is_empty() {
        return Err(anyhow!("No segments provided for rendering"));
    }

    let mut filter_complex = String::new();
    let n = segments.len();

    // 1. Trim and setpts for each segment
    for (i, seg) in segments.iter().enumerate() {
        let start = format!("{:.3}", seg.start);
        let end = format!("{:.3}", seg.end);
        
        if has_video {
            filter_complex.push_str(&format!(
                "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS[v{}]; ",
                start, end, i
            ));
        }
        filter_complex.push_str(&format!(
            "[0:a]atrim=start={}:end={},asetpts=PTS-STARTPTS[a{}]; ",
            start, end, i
        ));
    }

    // 2. Concat
    let mut concat_inputs = String::new();
    for i in 0..n {
        if has_video {
            concat_inputs.push_str(&format!("[v{}][a{}]", i, i));
        } else {
            concat_inputs.push_str(&format!("[a{}]", i));
        }
    }
    
    if has_video {
        filter_complex.push_str(&format!(
            "{}concat=n={}:v=1:a=1[vout][aout]; ",
            concat_inputs, n
        ));
        
        // 3. Post-processing on [vout]
        let mut final_v_filter = "[vout]split[a][b];[a]scale=-1:1920,crop=1080:1920,boxblur=20:20[bg];[b]scale=1080:-1[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2".to_string();
        if let Some(drawtext) = drawtext_filters {
            if !drawtext.is_empty() {
                final_v_filter = format!("{},{}", final_v_filter, drawtext);
            }
        }
        filter_complex.push_str(&format!("{}[final_v]", final_v_filter));
        
        cmd.args(["-filter_complex", &filter_complex]);
        cmd.args(["-map", "[final_v]", "-map", "[aout]"]);
        cmd.args(["-c:v", "libx264", "-preset", "fast", "-crf", "18", "-pix_fmt", "yuv420p"]);
    } else {
        filter_complex.push_str(&format!(
            "{}concat=n={}:v=0:a=1[aout]",
            concat_inputs, n
        ));
        cmd.args(["-filter_complex", &filter_complex]);
        cmd.args(["-map", "[aout]"]);
        cmd.arg("-vn");
    }

    cmd.args(["-c:a", "aac", "-b:a", "192k"]);
    cmd.arg(output_path);

    let output = cmd.output().context("running ffmpeg clip render")?;

    if !output.status.success() {
        return Err(anyhow!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(output_path.to_path_buf())
}
