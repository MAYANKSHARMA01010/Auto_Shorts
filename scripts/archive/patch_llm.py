import re

with open('src-tauri/src/llm.rs', 'r') as f:
    content = f.read()

# 1. Inject get_client
client_fn = r'''
fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}
'''
content = re.sub(r'(use serde_json::\{json, Value\};\n)', r'\1' + client_fn + r'\n', content)

# Replace reqwest::Client::new() with get_client()
content = content.replace('reqwest::Client::new()', 'get_client()')

# 2. Fix JSON extraction
old_json_extraction1 = r'''    let start = raw_response.find('{').unwrap_or(0);
    let end = raw_response.rfind('}').unwrap_or(raw_response.len() - 1);
    let json_str = if start <= end { &raw_response[start..=end] } else { &raw_response };'''

new_json_extraction1 = r'''    let start = raw_response.find('{');
    let end = raw_response.rfind('}');
    let json_str = match (start, end) {
        (Some(s), Some(e)) if s <= e => &raw_response[s..=e],
        _ => &raw_response,
    };'''
content = content.replace(old_json_extraction1, new_json_extraction1)

with open('src-tauri/src/llm.rs', 'w') as f:
    f.write(content)
print("Patched llm.rs")
