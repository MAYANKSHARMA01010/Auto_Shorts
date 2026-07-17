import re

with open('src-tauri/src/transcription.rs', 'r') as f:
    content = f.read()

# 1. Inject get_client
client_fn = r'''
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
'''
content = re.sub(r'(use tokio::fs;\n)', r'\1' + client_fn + r'\n', content)

# Replace reqwest::Client::new() with get_client()
content = content.replace('reqwest::Client::new()', 'get_client()')

with open('src-tauri/src/transcription.rs', 'w') as f:
    f.write(content)
print("Patched transcription.rs")
