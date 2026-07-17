import re

with open('src-tauri/src/llm.rs', 'r') as f:
    content = f.read()

new_functions = r'''
// ─── Viral Prediction ─────────────────────────────────────────────────────────────────

fn build_prediction_prompt(candidate_text: &str) -> String {
    format!(
r#"You are a world-class YouTube Shorts, Instagram Reels, and TikTok growth strategist.

Your job is NOT to edit the clip.
Your job is to predict how this clip is likely to perform with real viewers.

Analyze the clip from a viewer psychology perspective.
Evaluate:
1. First 3-second hook
2. Curiosity gap
3. Emotional engagement
4. Information density
5. Entertainment value
6. Surprise factor
7. Storytelling quality
8. Shareability
9. Rewatch potential
10. Audience retention prediction

Predict:
- Average watch percentage
- Likelihood of viewers watching to the end
- Best target audience
- Best platform (YouTube Shorts, TikTok, Instagram)
- Best posting time
- Best title
- Best thumbnail text (if applicable)
- Best hashtags

Explain:
- Why this clip could go viral.
- Why it might fail.
- What single change would most improve its performance.

Finally return:
Viral Probability: 0-100
Confidence: 0-100

Return ONLY valid JSON exactly matching this schema:
{{
  "evaluation_summary": "A 2-3 sentence summary evaluating the 10 psychology points above.",
  "predictions": {{
    "avg_watch_percentage": "e.g. 75%",
    "watch_to_end_likelihood": "e.g. High",
    "best_target_audience": "e.g. Tech enthusiasts 18-35",
    "best_platform": "e.g. TikTok",
    "best_posting_time": "e.g. 6 PM EST",
    "best_title": "e.g. The truth about AI coding",
    "best_thumbnail_text": "e.g. AI is taking over?",
    "best_hashtags": ["#tech", "#ai", "#coding"]
  }},
  "explanations": {{
    "viral_reason": "It hooks the viewer immediately with a controversial claim.",
    "failure_reason": "Might be too niche for a general audience.",
    "single_improvement": "Add sound effects during the transition."
  }},
  "scores": {{
    "viral_probability": 85,
    "confidence": 90
  }}
}}

Transcript to evaluate:
{candidate_text}"#
    )
}

fn parse_prediction_json(json_str: &str, candidate_id: &str) -> Result<crate::models::ViralPrediction> {
    let root: serde_json::Value = serde_json::from_str(json_str)
        .context("Failed to parse LLM output as JSON")?;

    let evaluation_summary = root.get("evaluation_summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    let preds = root.get("predictions").ok_or_else(|| anyhow!("Missing 'predictions' object"))?;
    let avg_watch_percentage = preds.get("avg_watch_percentage").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let watch_to_end_likelihood = preds.get("watch_to_end_likelihood").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let best_target_audience = preds.get("best_target_audience").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let best_platform = preds.get("best_platform").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let best_posting_time = preds.get("best_posting_time").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let best_title = preds.get("best_title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let best_thumbnail_text = preds.get("best_thumbnail_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut best_hashtags = Vec::new();
    if let Some(tags) = preds.get("best_hashtags").and_then(|v| v.as_array()) {
        for t in tags {
            if let Some(ts) = t.as_str() {
                best_hashtags.push(ts.to_string());
            }
        }
    }

    let expls = root.get("explanations").ok_or_else(|| anyhow!("Missing 'explanations' object"))?;
    let viral_reason = expls.get("viral_reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let failure_reason = expls.get("failure_reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let single_improvement = expls.get("single_improvement").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let scores = root.get("scores").ok_or_else(|| anyhow!("Missing 'scores' object"))?;
    let viral_probability = scores.get("viral_probability").and_then(|v| v.as_i64()).unwrap_or(0);
    let confidence = scores.get("confidence").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(crate::models::ViralPrediction {
        id: uuid::Uuid::new_v4().to_string(),
        candidate_id: candidate_id.to_string(),
        evaluation_summary,
        avg_watch_percentage,
        watch_to_end_likelihood,
        best_target_audience,
        best_platform,
        best_posting_time,
        best_title,
        best_thumbnail_text,
        best_hashtags,
        viral_reason,
        failure_reason,
        single_improvement,
        viral_probability,
        confidence,
    })
}

pub async fn predict_viral(
    candidate_id: &str,
    candidate_text: &str,
    provider: &str,
    api_key: Option<&str>,
    model_name: Option<&str>,
) -> Result<crate::models::ViralPrediction> {
    let prompt = build_prediction_prompt(candidate_text);
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

    parse_prediction_json(json_str, candidate_id)
}
'''
content = content + new_functions

with open('src-tauri/src/llm.rs', 'w') as f:
    f.write(content)

print("Patched llm.rs for prediction")
