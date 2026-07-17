import re

with open('src-tauri/src/media.rs', 'r') as f:
    content = f.read()

# Add check in extract_audio
check_extract = r'''
    if !std::path::Path::new(source_path).exists() {
        return Err(anyhow!("Source video file does not exist: {}", source_path));
    }
'''
content = re.sub(r'(pub fn extract_audio\(source_path: &str, output_path: &str\) -> Result<\(\)> \{\n)', r'\1' + check_extract, content)

# Add check in render_clip
check_render = r'''
    if !std::path::Path::new(source_path).exists() {
        return Err(anyhow!("Source video file does not exist: {}", source_path));
    }
'''
content = re.sub(r'(    if !Command::new\("ffmpeg"\).arg\("-version"\).output\(\).is_ok\(\) \{\n        return Err\(anyhow!\("ffmpeg is not installed or not available on PATH"\)\);\n    \}\n)', r'\1' + check_render, content)

with open('src-tauri/src/media.rs', 'w') as f:
    f.write(content)
print("Patched media.rs")
