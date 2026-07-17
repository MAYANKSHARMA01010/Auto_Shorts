with open('src-tauri/src/llm.rs', 'r') as f:
    llm_content = f.read()
    
with open('src-tauri/src/transcription.rs', 'r') as f:
    trans_content = f.read()

client_fn = r'''
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
'''
if 'fn get_client()' not in llm_content:
    with open('src-tauri/src/llm.rs', 'w') as f:
        f.write(client_fn + '\n' + llm_content)

client_fn_t = r'''
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
'''
if 'fn get_client()' not in trans_content:
    with open('src-tauri/src/transcription.rs', 'w') as f:
        f.write(client_fn_t + '\n' + trans_content)

print("Fixed get_client")
