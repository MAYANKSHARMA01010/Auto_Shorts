fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::models::{NormalizedTranscript, TranscriptSegment, TranscriptWord};

pub async fn transcribe_deepgram(audio_path: &str, api_key: &str) -> Result<NormalizedTranscript> {
    let bytes = tokio::fs::read(audio_path)
        .await
        .with_context(|| format!("reading audio file {audio_path}"))?;

    let response = get_client()
        .post("https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&diarize=true&punctuate=true&filler_words=true")
        .header("Authorization", format!("Token {api_key}"))
        .header("Content-Type", "audio/wav")
        .body(bytes)
        .send()
        .await
        .context("calling Deepgram")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Deepgram request failed ({status}): {body}"));
    }

    let value: Value = response.json().await.context("parsing Deepgram response")?;
    normalize_deepgram(value)
}

fn normalize_deepgram(value: Value) -> Result<NormalizedTranscript> {
    let alternative = value
        .pointer("/results/channels/0/alternatives/0")
        .ok_or_else(|| anyhow!("Deepgram response did not include an alternative transcript"))?;

    let language = value
        .pointer("/metadata/language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_string();

    let duration = value
        .pointer("/metadata/duration")
        .and_then(Value::as_f64)
        .unwrap_or_default();

    let raw_words = alternative
        .get("words")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Deepgram response did not include word timestamps"))?;

    let mut speakers = BTreeSet::new();
    let mut words = Vec::with_capacity(raw_words.len());

    for word in raw_words {
        let text = word
            .get("punctuated_word")
            .or_else(|| word.get("word"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }

        let speaker = word
            .get("speaker")
            .and_then(Value::as_i64)
            .map(|speaker| format!("S{}", speaker + 1));
        if let Some(speaker) = &speaker {
            speakers.insert(speaker.clone());
        }

        words.push(TranscriptWord {
            text,
            start: word
                .get("start")
                .and_then(Value::as_f64)
                .unwrap_or_default(),
            end: word.get("end").and_then(Value::as_f64).unwrap_or_default(),
            speaker,
        });
    }

    let segments = build_segments(&words);

    Ok(NormalizedTranscript {
        language,
        duration,
        speakers: speakers.into_iter().collect(),
        words,
        segments,
    })
}

pub fn build_segments(words: &[TranscriptWord]) -> Vec<TranscriptSegment> {
    let mut segments = Vec::new();
    let mut current: Option<TranscriptSegment> = None;

    for word in words {
        let should_break = current.as_ref().is_some_and(|segment| {
            let pause = word.start - segment.end;
            let speaker_changed = segment.speaker != word.speaker;
            let sentence_end = segment.text.ends_with(['.', '!', '?']);
            pause > 0.9 || speaker_changed || sentence_end
        });

        if should_break {
            if let Some(segment) = current.take() {
                segments.push(segment);
            }
        }

        match &mut current {
            Some(segment) => {
                segment.end = word.end;
                segment.text.push(' ');
                segment.text.push_str(&word.text);
            }
            None => {
                current = Some(TranscriptSegment {
                    start: word.start,
                    end: word.end,
                    speaker: word.speaker.clone(),
                    text: word.text.clone(),
                    audio_metadata: None,
                    visual_metadata: None,
                });
            }
        }
    }

    if let Some(segment) = current {
        segments.push(segment);
    }

    segments
}

pub fn whisper_cli_exists() -> bool {
    std::process::Command::new("whisper")
        .arg("--help")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn whisper_python_exists() -> bool {
    std::process::Command::new("python3")
        .args(["-c", "import whisper"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn normalize_whisper_raw_json(raw: serde_json::Value) -> Result<NormalizedTranscript> {
    let language = raw
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("en")
        .to_string();

    let segments_arr = raw
        .get("segments")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing 'segments' in Whisper JSON"))?;

    let duration = segments_arr
        .last()
        .and_then(|s| s.get("end").and_then(|e| e.as_f64()))
        .unwrap_or(0.0);

    let mut segments = Vec::new();
    let mut words = Vec::new();

    for seg in segments_arr {
        let start = seg.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let end = seg.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let text = seg
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        segments.push(TranscriptSegment {
            start,
            end,
            speaker: Some("S1".to_string()),
            text,
            audio_metadata: None,
            visual_metadata: None,
        });

        if let Some(words_arr) = seg.get("words").and_then(|v| v.as_array()) {
            for w in words_arr {
                let word_text = w
                    .get("word")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let word_start = w.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let word_end = w.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);

                words.push(TranscriptWord {
                    text: word_text,
                    start: word_start,
                    end: word_end,
                    speaker: Some("S1".to_string()),
                });
            }
        }
    }

    Ok(NormalizedTranscript {
        language,
        duration,
        speakers: vec!["S1".to_string()],
        words,
        segments,
    })
}

pub async fn transcribe_local_with_progress(
    audio_path: &str,
    model_path: &str,
    duration_sec: f64,
    on_progress: Option<Box<dyn Fn(f64) + Send>>,
) -> Result<NormalizedTranscript> {
    let audio_path = audio_path.to_string();
    let model_path = model_path.to_string();

    if whisper_cli_exists() {
        let audio_path_buf = std::path::Path::new(&audio_path);
        let audio_dir = audio_path_buf
            .parent()
            .ok_or_else(|| anyhow!("Invalid audio path parent"))?;
        let audio_stem = audio_path_buf
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid audio file stem"))?
            .to_string_lossy();

        let output_json_path = audio_dir.join(format!("{}.json", audio_stem));
        let output_json_path_str = output_json_path.to_string_lossy().to_string();
        let audio_dir_str = audio_dir.to_string_lossy().to_string();
        let audio_dir_clone = audio_dir.to_path_buf();
        let audio_stem_clone = audio_stem.to_string();

        tokio::task::spawn_blocking(move || {
            let mut child = std::process::Command::new("whisper")
                .env("PYTHONUNBUFFERED", "1")
                .arg(&audio_path)
                .args(["--model", "base"])
                .args(["--output_format", "json"])
                .args(["--output_dir", &audio_dir_str])
                .args(["--word_timestamps", "True"])
                .args(["--verbose", "True"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context("executing whisper CLI")?;

            if let (Some(stdout), Some(ref on_prog)) = (child.stdout.take(), on_progress) {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Look for "[00:00.000 --> 00:05.000]"
                    if line.starts_with('[') {
                        if let Some(arrow_idx) = line.find("-->") {
                            let end_part = &line[arrow_idx + 3..].trim();
                            if let Some(end_bracket) = end_part.find(']') {
                                let time_str = &end_part[..end_bracket];
                                let parts: Vec<&str> = time_str.split(':').collect();
                                if parts.len() >= 2 {
                                    let mut sec = 0.0;
                                    let mut mult = 1.0;
                                    for p in parts.iter().rev() {
                                        sec += p.parse::<f64>().unwrap_or(0.0) * mult;
                                        mult *= 60.0;
                                    }
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

            let output = child
                .wait_with_output()
                .context("waiting for whisper CLI")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                return Err(anyhow!(
                    "Whisper CLI failed:\nStderr: {}\nStdout: {}",
                    stderr,
                    stdout
                ));
            }

            let json_bytes = std::fs::read(&output_json_path_str)
                .context("reading output transcript JSON from CLI")?;
            let raw_json: serde_json::Value =
                serde_json::from_slice(&json_bytes).context("parsing output transcript JSON")?;

            // Clean up the output JSON file
            let _ = std::fs::remove_file(&output_json_path_str);

            // Clean up any extra formats whisper CLI might have written (it sometimes generates them by default)
            for ext in &["txt", "srt", "vtt", "tsv"] {
                let extra_file = audio_dir_clone.join(format!("{}.{}", audio_stem_clone, ext));
                if extra_file.exists() {
                    let _ = std::fs::remove_file(extra_file);
                }
            }

            normalize_whisper_raw_json(raw_json)
        })
        .await
        .context("spawn_blocking failed")?
    } else {
        // Resolve the directory where the model lives. We'll put transcribe.py there.
        let model_dir = std::path::Path::new(&model_path)
            .parent()
            .ok_or_else(|| anyhow!("Invalid model path"))?;

        let script_path = model_dir.join("transcribe.py");
        // Always rewrite to ensure we have the latest pipeline
        let script_content = r#"import sys
import json
import os

FASTER_WHISPER_AVAILABLE = False
WHISPERX_AVAILABLE = False

try:
    from faster_whisper import WhisperModel
    FASTER_WHISPER_AVAILABLE = True
except ImportError:
    pass

try:
    import whisperx
    WHISPERX_AVAILABLE = True
except ImportError:
    pass


def format_timestamp(seconds):
    """Format seconds into [HH:MM:SS.mmm] style for progress parsing."""
    h = int(seconds // 3600)
    m = int((seconds % 3600) // 60)
    s = seconds % 60
    return f"[{h:02d}:{m:02d}.{s:06.3f}]"


def run_faster_whisper_pipeline(audio_path, output_json_path, duration_hint, device):
    """High quality pipeline: faster-whisper large-v3 + WhisperX forced alignment."""
    import torch
    
    compute_type = "float16" if device == "cuda" else "int8"
    
    print(f"[Pipeline] Loading faster-whisper large-v3 on {device}...", flush=True)
    model = WhisperModel("large-v3", device=device, compute_type=compute_type)
    
    # Transcribe - stream segment progress
    segments_list = []
    segments_gen, info = model.transcribe(
        audio_path,
        word_timestamps=True,
        vad_filter=True,
        vad_parameters={"min_silence_duration_ms": 500},
    )
    
    language = info.language
    audio_duration = info.duration if hasattr(info, "duration") else duration_hint

    for segment in segments_gen:
        segments_list.append(segment)
        # Emit progress in format that Rust parser understands: "[MM:SS.mmm --> MM:SS.mmm]"
        end_sec = segment.end
        if audio_duration and audio_duration > 0:
            h = int(end_sec // 3600)
            m = int((end_sec % 3600) // 60)
            s = end_sec % 60
            print(f"[{h:02d}:{m:06.3f} --> {h:02d}:{m:06.3f}] {segment.text.strip()}", flush=True)

    if not segments_list:
        return None

    # Build initial word list from faster-whisper
    words_fw = []
    for seg in segments_list:
        if seg.words:
            for w in seg.words:
                words_fw.append({
                    "text": w.word.strip(),
                    "start": w.start,
                    "end": w.end,
                    "speaker": "S1"
                })

    # WhisperX forced alignment for better word-level sync
    aligned_words = words_fw  # fallback to faster-whisper words
    if WHISPERX_AVAILABLE:
        try:
            import torch
            print("[Pipeline] Running WhisperX forced alignment...", flush=True)
            wx_audio = whisperx.load_audio(audio_path)
            align_model, metadata = whisperx.load_align_model(
                language_code=language, device=device
            )
            # Build whisperx-compatible transcript format
            wx_segments = [
                {
                    "start": seg.start,
                    "end": seg.end,
                    "text": seg.text.strip(),
                    "words": [
                        {"word": w.word.strip(), "start": w.start, "end": w.end, "score": getattr(w, "probability", 1.0)}
                        for w in (seg.words or [])
                    ]
                }
                for seg in segments_list
            ]
            result_aligned = whisperx.align(
                wx_segments, align_model, metadata, wx_audio, device,
                return_char_alignments=False
            )
            aligned_words = []
            for seg in result_aligned.get("segments", []):
                for w in seg.get("words", []):
                    if "word" in w and "start" in w and "end" in w:
                        aligned_words.append({
                            "text": w["word"].strip(),
                            "start": w["start"],
                            "end": w["end"],
                            "speaker": "S1"
                        })
            print("[Pipeline] Alignment complete.", flush=True)
        except Exception as e:
            print(f"[Pipeline] WhisperX alignment failed, using faster-whisper timestamps: {e}", flush=True)
            aligned_words = words_fw

    # Build segments from aligned words
    segments_out = []
    if aligned_words:
        current_words = []
        for word in aligned_words:
            current_words.append(word)
            # Group into ~10 word segments
            text_so_far = " ".join(w["text"] for w in current_words)
            if len(current_words) >= 10 or word["text"].endswith((".", "!", "?")):
                segments_out.append({
                    "start": current_words[0]["start"],
                    "end": current_words[-1]["end"],
                    "speaker": "S1",
                    "text": text_so_far
                })
                current_words = []
        if current_words:
            text_so_far = " ".join(w["text"] for w in current_words)
            segments_out.append({
                "start": current_words[0]["start"],
                "end": current_words[-1]["end"],
                "speaker": "S1",
                "text": text_so_far
            })
    else:
        # Fallback: use faster-whisper segments directly
        for seg in segments_list:
            segments_out.append({
                "start": seg.start,
                "end": seg.end,
                "speaker": "S1",
                "text": seg.text.strip()
            })

    duration = aligned_words[-1]["end"] if aligned_words else (segments_out[-1]["end"] if segments_out else 0.0)

    normalized = {
        "language": language,
        "duration": duration,
        "speakers": ["S1"],
        "words": aligned_words,
        "segments": segments_out,
        "pipeline": "faster-whisper+whisperx" if WHISPERX_AVAILABLE else "faster-whisper"
    }
    return normalized


def run_openai_whisper_pipeline(audio_path, output_json_path, model_name):
    """Fallback pipeline using openai-whisper."""
    import whisper
    print(f"[Pipeline] Falling back to openai-whisper {model_name}...", flush=True)
    model = whisper.load_model(model_name)
    result = model.transcribe(audio_path, word_timestamps=True, verbose=True)

    words_out = []
    segments_out = []
    for segment in result.get("segments", []):
        segments_out.append({
            "start": segment["start"],
            "end": segment["end"],
            "speaker": "S1",
            "text": segment["text"].strip()
        })
        for word in segment.get("words", []):
            words_out.append({
                "text": word["word"].strip(),
                "start": word["start"],
                "end": word["end"],
                "speaker": "S1"
            })

    duration = segments_out[-1]["end"] if segments_out else 0.0
    normalized = {
        "language": result.get("language", "en"),
        "duration": duration,
        "speakers": ["S1"],
        "words": words_out,
        "segments": segments_out,
        "pipeline": "openai-whisper"
    }
    return normalized


def main():
    if len(sys.argv) < 3:
        print("Usage: transcribe.py <audio_path> <output_json_path> [model_name]")
        sys.exit(1)

    audio_path = sys.argv[1]
    output_json_path = sys.argv[2]
    model_name = sys.argv[3] if len(sys.argv) > 3 else "base"

    # Detect best available device
    try:
        import torch
        if torch.cuda.is_available():
            device = "cuda"
        elif hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
            device = "mps"
        else:
            device = "cpu"
    except ImportError:
        device = "cpu"

    normalized = None

    if FASTER_WHISPER_AVAILABLE:
        try:
            normalized = run_faster_whisper_pipeline(audio_path, output_json_path, 0.0, device if device != "mps" else "cpu")
        except Exception as e:
            print(f"[Pipeline] faster-whisper failed: {e}. Falling back...", flush=True)

    if normalized is None:
        try:
            normalized = run_openai_whisper_pipeline(audio_path, output_json_path, model_name)
        except Exception as e:
            print(f"[Pipeline] openai-whisper also failed: {e}", flush=True)
            sys.exit(1)

    with open(output_json_path, "w", encoding="utf-8") as f:
        json.dump(normalized, f, indent=2, ensure_ascii=False)

    print(f"[Pipeline] Done. Written to {output_json_path}", flush=True)


if __name__ == "__main__":
    main()
"#;
        std::fs::write(&script_path, script_content).context("writing transcribe.py script")?;

        let output_json_path =
            model_dir.join(format!("temp_transcript_{}.json", uuid::Uuid::new_v4()));
        let script_path_str = script_path.to_string_lossy().to_string();
        let output_json_path_str = output_json_path.to_string_lossy().to_string();

        tokio::task::spawn_blocking(move || {
            // Check for a local virtual environment first (better for portability)
            let possible_venvs = vec![
                ".venv/bin/python3",
                "../.venv/bin/python3",
                "venv/bin/python3",
                "../venv/bin/python3",
                "/opt/homebrew/lib/python3.12/autoshorts-venv/bin/python3.12", // legacy fallback
            ];

            let mut python_cmd = "python3".to_string(); // fallback to system python
            for venv_path in possible_venvs {
                if std::path::Path::new(venv_path).exists() {
                    python_cmd = venv_path.to_string();
                    break;
                }
            }

            let mut child = std::process::Command::new(&python_cmd)
                .env("PYTHONUNBUFFERED", "1")
                .arg(&script_path_str)
                .arg(&audio_path)
                .arg(&output_json_path_str)
                .arg("base") // default model size (overridden by faster-whisper inside script)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context("executing python transcribe.py")?;

            if let (Some(stdout), Some(ref on_prog)) = (child.stdout.take(), on_progress) {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Look for "[00:00.000 --> 00:05.000]"
                    if line.starts_with('[') {
                        if let Some(arrow_idx) = line.find("-->") {
                            let end_part = &line[arrow_idx + 3..].trim();
                            if let Some(end_bracket) = end_part.find(']') {
                                let time_str = &end_part[..end_bracket];
                                let parts: Vec<&str> = time_str.split(':').collect();
                                if parts.len() >= 2 {
                                    let mut sec = 0.0;
                                    let mut mult = 1.0;
                                    for p in parts.iter().rev() {
                                        sec += p.parse::<f64>().unwrap_or(0.0) * mult;
                                        mult *= 60.0;
                                    }
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

            let output = child
                .wait_with_output()
                .context("waiting for python script")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                return Err(anyhow!(
                    "Python transcription script failed:\nStderr: {}\nStdout: {}",
                    stderr,
                    stdout
                ));
            }

            let json_bytes =
                std::fs::read(&output_json_path_str).context("reading output transcript JSON")?;
            let transcript: NormalizedTranscript =
                serde_json::from_slice(&json_bytes).context("parsing output transcript JSON")?;

            // Cleanup temp file
            let _ = std::fs::remove_file(&output_json_path_str);

            Ok(transcript)
        })
        .await
        .context("spawn_blocking failed")?
    }
}
pub fn parse_vtt(content: &str) -> Result<NormalizedTranscript> {
    let mut segments: Vec<crate::models::TranscriptSegment> = Vec::new();
    let mut words = Vec::new();
    let mut duration = 0.0;

    for block in content.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let mut text_lines = Vec::new();
        let mut start_sec = 0.0;
        let mut end_sec = 0.0;
        let mut found_ts = false;

        for line in lines {
            if line.contains("-->") {
                let parts: Vec<&str> = line.split("-->").collect();
                if parts.len() == 2 {
                    start_sec = parse_vtt_timestamp(parts[0].trim());
                    // Support VTT style .mss and SRT style ,mss
                    let end_str = parts[1].split_whitespace().next().unwrap_or(parts[1]);
                    end_sec = parse_vtt_timestamp(end_str.trim());
                    found_ts = true;
                }
            } else if found_ts {
                text_lines.push(line.trim());
            }
        }

        if found_ts && !text_lines.is_empty() {
            let raw_text = text_lines.join(" ").replace("\n", " ");

            // Strip HTML/VTT tags like <c.colorA0AAB4>, <b>, and <00:00:00.444>
            let mut text = String::with_capacity(raw_text.len());
            let mut in_tag = false;
            for c in raw_text.chars() {
                if c == '<' {
                    in_tag = true;
                } else if c == '>' {
                    in_tag = false;
                } else if !in_tag {
                    text.push(c);
                }
            }
            let text = text.trim().to_string();

            if text.is_empty() {
                continue;
            }

            // Deduplicate consecutive identical segments
            if let Some(last) = segments.last_mut() {
                if last.text == text {
                    // Update end time
                    last.end = end_sec.max(last.end);
                    if end_sec > duration {
                        duration = end_sec;
                    }
                    continue;
                }
            }

            segments.push(TranscriptSegment {
                start: start_sec,
                end: end_sec,
                text: text.clone(),
                speaker: None,
                audio_metadata: None,
                visual_metadata: None,
            });
            if end_sec > duration {
                duration = end_sec;
            }
        }
    }

    // Generate words array from deduplicated segments
    for seg in &segments {
        let raw_words: Vec<&str> = seg.text.split_whitespace().collect();
        if !raw_words.is_empty() {
            let dur = seg.end - seg.start;
            let time_per_word = dur / (raw_words.len() as f64);

            for (i, w) in raw_words.iter().enumerate() {
                let w_start = seg.start + (i as f64 * time_per_word);
                let w_end = seg.start + ((i + 1) as f64 * time_per_word);
                words.push(TranscriptWord {
                    start: w_start,
                    end: w_end,
                    text: w.to_string(),
                    speaker: None,
                });
            }
        }
    }

    Ok(NormalizedTranscript {
        language: "en".to_string(),
        duration,
        speakers: vec![],
        segments,
        words,
    })
}

fn parse_vtt_timestamp(ts: &str) -> f64 {
    let clean_ts = ts.replace(',', ".");
    let parts: Vec<&str> = clean_ts.split(':').collect();
    let mut seconds = 0.0;

    if parts.len() == 3 {
        if let Ok(h) = parts[0].parse::<f64>() {
            seconds += h * 3600.0;
        }
        if let Ok(m) = parts[1].parse::<f64>() {
            seconds += m * 60.0;
        }
        if let Ok(s) = parts[2].parse::<f64>() {
            seconds += s;
        }
    } else if parts.len() == 2 {
        if let Ok(m) = parts[0].parse::<f64>() {
            seconds += m * 60.0;
        }
        if let Ok(s) = parts[1].parse::<f64>() {
            seconds += s;
        }
    }

    seconds
}
