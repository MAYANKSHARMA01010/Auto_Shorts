import re

with open('src-tauri/src/llm.rs', 'r') as f:
    content = f.read()

# Replace the current review_candidate with one that uses the raw functions
old_review = r'''pub async fn review_candidate\(
    candidate_id: &str,
    candidate_text: &str,
    provider: &str,
    api_key: Option<&str>,
    model_name: Option<&str>,
\) -> Result<crate::models::CandidateReview> \{
    let prompt = build_review_prompt\(candidate_text\);
    let api_key = api_key\.unwrap_or\(""\);
    let raw_response = generate_llm_completion\(&prompt, provider, api_key, model_name\)\.await\?;
    let json_str = extract_json_block\(&raw_response\);
    parse_review_json\(&json_str, candidate_id\)
\}'''

new_review = r'''pub async fn review_candidate(
    candidate_id: &str,
    candidate_text: &str,
    provider: &str,
    api_key: Option<&str>,
    model_name: Option<&str>,
) -> Result<crate::models::CandidateReview> {
    let prompt = build_review_prompt(candidate_text);
    let key = api_key.unwrap_or("");
    let model = model_name.unwrap_or("");

    let raw_response = match provider {
        "deepseek" => call_deepseek_raw(&prompt, key).await?,
        "claude" => call_claude_raw(&prompt, key).await?,
        "gemini" => call_gemini_raw(&prompt, key).await?,
        "openai" => call_openai_raw(&prompt, key, model).await?,
        "openrouter" => call_openrouter_raw(&prompt, key, model).await?,
        "local" => call_ollama_raw(&prompt, model).await?,
        _ => return Err(anyhow::anyhow!("Unsupported provider: {}", provider)),
    };

    let start = raw_response.find('{').unwrap_or(0);
    let end = raw_response.rfind('}').unwrap_or(raw_response.len() - 1);
    let json_str = if start <= end { &raw_response[start..=end] } else { &raw_response };

    parse_review_json(json_str, candidate_id)
}'''

content = re.sub(old_review, new_review, content)

with open('src-tauri/src/llm.rs', 'w') as f:
    f.write(content)

print("Patched llm.rs review_candidate")
