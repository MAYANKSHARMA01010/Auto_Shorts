fn get_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::{CandidateDraft, NormalizedTranscript, TranscriptSegment};

fn build_prompt(segments: &str) -> String {
    format!(
        r#"You are an expert short-form video editor for YouTube Shorts, Instagram Reels, and TikTok.
Your goal is to create the most engaging, complete, and natural story possible. You are NOT a timestamp finder; you are creating a brand new edit. The transcript represents raw footage.

==================================================
EDITOR PHILOSOPHY & PRINCIPLES
==================================================
- Every second must earn its place. If something does not increase curiosity, emotion, understanding, or move the story forward—remove it.
- Never remove context, setup, or payoff unless they are genuinely unnecessary.
- The AI should NEVER assume that one continuous clip is best. You may trim, remove, merge, stitch, compress, shorten, and combine multiple transcript regions to produce the strongest possible story.
- There is no preferred editing style. Only viewer retention matters.
- A clip MUST be at least 30 seconds long.
- A complete story with a slightly lower viral potential is ALWAYS better than an incomplete story with a higher viral score.

==================================================
EDITING STYLES & DIVERSITY
==================================================
Choose the structure that maximizes retention. Possible editing styles include:
- continuous clip
- stitched edit
- montage
- before -> after
- problem -> solution
- setup -> payoff
- question -> answer
- challenge -> resolution
- explanation -> demonstration

Candidate Diversity: Do not generate near-duplicate candidates. Each candidate should represent a genuinely different editing strategy whenever appropriate (e.g., Story-focused vs. High-retention fast pacing vs. Educational clarity).

==================================================
STITCHING RULES & PACE OPTIMIZATION
==================================================
- Feel free to take 12 seconds from minute 2, combine with 18 seconds from minute 7, and 15 seconds from minute 14 if together they produce the strongest story.
- Never destroy chronology unless explicitly supported by the source material. Never create misleading narratives or fabricate meaning.
- Constantly ask: "Would the average Shorts viewer swipe away here?" If yes: remove, compress, or jump forward.
- CRITICAL: To make a clip long enough, you MUST combine multiple consecutive transcript lines into a single segment. Use the start time of the FIRST line and the end time of the LAST line to form a long segment. Do NOT just copy a single 2-second line.

==================================================
MULTIMODAL HEURISTICS (AUDIO & VISUAL SIGNALS)
==================================================
You will receive [Aud: ...] and [Vis: ...] metadata tags attached to transcript segments.
Use these signals to make better editing decisions:
- **Scene Changes**: Prefer beginning a clip immediately after a scene change (e.g. `SceneChange=True`).
- **Silence**: Avoid cutting during silence (`isSilence=True`) unless intentionally pacing a dramatic pause.
- **Visual Energy**: Prefer cuts when visual energy or motion increases (`Motion=high`). Maintain visual continuity.
- **Faces**: Prioritize retaining segments where faces are present (`Face=True`).
These are heuristics, not rigid rules. The narrative story is always the priority.

==================================================
INTERNAL MULTI-PASS REASONING
==================================================
Perform the following reasoning internally before producing your answer. Do NOT reveal your reasoning. Return ONLY the required JSON.
- Pass 1: Understand transcript
- Pass 2: Identify content type (e.g., Storytelling, Tutorial, Comedy)
- Pass 3: Identify emotional peaks and hooks
- Pass 4: Group related moments
- Pass 5: Choose editing strategy (e.g., problem -> solution, montage)
- Pass 6: Construct complete story
- Pass 7: Optimize pacing
- Pass 8: Merge nearby segments
- Pass 9: Validate final edit

==================================================
OPTIONAL METADATA
==================================================
You may optionally output your chosen `editing_strategy` (e.g. "problem -> solution") and a `confidence` score (e.g. 0.95) for each candidate.

Return ONLY valid JSON strictly in the following schema:
{{
  "candidates": [
    {{
      "score": 9.7,
      "hook": "The first spoken sentence that hooks the viewer",
      "rationale": "Why this edit works well for viral short form",
      "editing_strategy": "problem -> solution",
      "confidence": 0.92,
      "segments": [
        {{
          "start": 12.5,
          "end": 45.2,
          "text": "The exact transcript text here"
        }}
      ]
    }}
  ]
}}

Transcript:
{segments}"#,
        segments = segments
    )
}

#[derive(Debug, Deserialize)]
struct AnthropicMessage {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeepseekMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepseekChoice {
    message: DeepseekMessage,
}

#[derive(Debug, Deserialize)]
struct DeepseekResponse {
    choices: Vec<DeepseekChoice>,
}

pub async fn detect_candidates_with_deepseek(
    transcript: &NormalizedTranscript,
    api_key: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let prompt = build_prompt(&segments);

    let response = get_client()
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "temperature": 0.2,
            "response_format": {
                "type": "json_object"
            }
        }))
        .send()
        .await
        .context("calling DeepSeek")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("DeepSeek request failed ({status}): {body}"));
    }

    let res_body: DeepseekResponse = response.json().await.context("parsing DeepSeek response")?;
    let text = res_body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("DeepSeek response did not include choices content"))?;

    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&text, min_duration)
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

pub async fn detect_candidates_with_gemini(
    transcript: &NormalizedTranscript,
    api_key: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let prompt = build_prompt(&segments);

    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = get_client()
        .post(&url)
        .json(&json!({
            "contents": [
                {
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "temperature": 0.2
            }
        }))
        .send()
        .await
        .context("calling Gemini")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Gemini request failed ({status}): {body}"));
    }

    let res_body: GeminiResponse = response.json().await.context("parsing Gemini response")?;
    let text = res_body
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .and_then(|p| p.text.clone())
        .ok_or_else(|| anyhow!("Gemini response did not include content text"))?;

    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&text, min_duration)
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: String,
}

pub async fn detect_candidates_with_openai(
    transcript: &NormalizedTranscript,
    api_key: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let prompt = build_prompt(&segments);

    let response = get_client()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "temperature": 0.2,
            "response_format": {
                "type": "json_object"
            }
        }))
        .send()
        .await
        .context("calling OpenAI")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("OpenAI request failed ({status}): {body}"));
    }

    let res_body: ChatCompletionResponse =
        response.json().await.context("parsing OpenAI response")?;
    let text = res_body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("OpenAI response did not include choices content"))?;

    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&text, min_duration)
}

pub async fn detect_candidates_with_openrouter(
    transcript: &NormalizedTranscript,
    api_key: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let model =
        std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "google/gemini-2.5-flash".to_string());
    let prompt = build_prompt(&segments);

    let response = get_client()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "temperature": 0.2,
            "response_format": {
                "type": "json_object"
            }
        }))
        .send()
        .await
        .context("calling OpenRouter")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("OpenRouter request failed ({status}): {body}"));
    }

    let res_body: ChatCompletionResponse = response
        .json()
        .await
        .context("parsing OpenRouter response")?;
    let text = res_body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow!("OpenRouter response did not include choices content"))?;

    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&text, min_duration)
}

#[derive(Debug, Serialize)]
struct ClaudeMessage<'a> {
    role: &'a str,
    content: String,
}

pub async fn detect_candidates_with_claude(
    transcript: &NormalizedTranscript,
    api_key: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let prompt = build_prompt(&segments);

    let model =
        std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-5-sonnet-latest".to_string());

    let response = get_client()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": model,
            "max_tokens": 1800,
            "temperature": 0.2,
            "messages": [
                ClaudeMessage {
                    role: "user",
                    content: prompt,
                }
            ]
        }))
        .send()
        .await
        .context("calling Claude")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Claude request failed ({status}): {body}"));
    }

    let message: AnthropicMessage = response.json().await.context("parsing Claude response")?;
    let text = message
        .content
        .into_iter()
        .find_map(|content| content.text)
        .ok_or_else(|| anyhow!("Claude response did not include text content"))?;

    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&text, min_duration)
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

pub async fn detect_candidates_with_local_llm(
    transcript: &NormalizedTranscript,
    model_name: &str,
) -> Result<Vec<CandidateDraft>> {
    let segments = compact_segments(&transcript.segments);
    let prompt = build_prompt(&segments);

    let response = get_client()
        .post("http://localhost:11434/api/chat")
        .json(&json!({
            "model": model_name,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "stream": false,
            "options": {
                "temperature": 0.2
            },
            "format": {
                "type": "object",
                "properties": {
                    "candidates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "score": { "type": "number" },
                                "hook": { "type": "string" },
                                "rationale": { "type": "string" },
                                "editing_strategy": { "type": "string" },
                                "confidence": { "type": "number" },
                                "segments": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "start": { "type": "number" },
                                            "end": { "type": "number" },
                                            "text": { "type": "string" }
                                        },
                                        "required": ["start", "end", "text"]
                                    }
                                }
                            },
                            "required": ["score", "hook", "rationale", "segments"]
                        }
                    }
                },
                "required": ["candidates"]
            }
        }))
        .send()
        .await
        .context("calling local Ollama")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("Local Ollama request failed ({status}): {body}"));
    }

    let res_body: OllamaResponse = response
        .json()
        .await
        .context("parsing local Ollama response")?;

    println!("RAW OLLAMA OUTPUT: {}", res_body.message.content);

    // Make min_duration much more forgiving (10s minimum for normal videos, 5s for tiny videos)
    let min_duration = if transcript.duration < 15.0 { 1.5 } else { 2.0 };
    parse_candidate_json(&res_body.message.content, min_duration)
}

fn compact_segments(segments: &[TranscriptSegment]) -> String {
    segments
        .iter()
        .map(|segment| {
            let speaker = segment.speaker.as_deref().unwrap_or("Speaker");
            
            let mut meta_str = String::new();
            if let Some(vis) = &segment.visual_metadata {
                meta_str.push_str(&format!(
                    "[Vis: SceneChange={}, Motion={}, Face={}] ",
                    vis.scene_change.unwrap_or(false),
                    vis.motion_level.as_deref().unwrap_or("low"),
                    vis.face_present.unwrap_or(false)
                ));
            }
            if let Some(aud) = &segment.audio_metadata {
                meta_str.push_str(&format!(
                    "[Aud: Silence={}, Vol={:.1}dB] ",
                    aud.is_silence.unwrap_or(false),
                    aud.average_volume.unwrap_or(-100.0)
                ));
            }
            
            format!(
                "[{:.2}-{:.2}] {}{}: {}",
                segment.start, segment.end, meta_str, speaker, segment.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_candidate_json(text: &str, min_duration: f64) -> Result<Vec<CandidateDraft>> {
    let trimmed = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let val: serde_json::Value = serde_json::from_str(trimmed).context("parsing candidate JSON")?;

    let candidates_arr = if val.is_array() {
        val.as_array().cloned()
    } else if val.is_object() {
        let mut found_arr = None;
        for key in &[
            "candidates",
            "Candidates",
            "moments",
            "clips",
            "segments",
            "results",
        ] {
            if let Some(arr) = val.get(*key).and_then(|v| v.as_array()) {
                found_arr = Some(arr.clone());
                break;
            }
        }
        found_arr
    } else {
        None
    };

    let concrete_arr = candidates_arr.ok_or_else(|| {
        anyhow!(
            "LLM output does not contain a candidates array. Raw output: {}",
            trimmed
        )
    })?;

    let mut drafts = Vec::new();
    for item in &concrete_arr {
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = item
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut score = match item.get("score").or_else(|| item.get("viral_score")) {
            Some(v) => {
                if let Some(f) = v.as_f64() {
                    f
                } else if let Some(s) = v.as_str() {
                    s.parse::<f64>().unwrap_or(0.8)
                } else if let Some(i) = v.as_i64() {
                    i as f64
                } else {
                    0.8
                }
            }
            None => 0.8,
        };

        if score > 1.0 && score <= 10.0 {
            score /= 10.0;
        } else if score > 10.0 && score <= 100.0 {
            score /= 100.0;
        } else if score > 100.0 {
            score = 1.0;
        } else if score < 0.0 {
            score = 0.0;
        }

        let hook = item
            .get("hook")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let rationale = item
            .get("rationale")
            .or_else(|| item.get("reason"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let editing_strategy = item
            .get("editing_strategy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let confidence = item.get("confidence").and_then(|v| {
            if let Some(f) = v.as_f64() {
                Some(f)
            } else if let Some(s) = v.as_str() {
                s.parse::<f64>().ok()
            } else {
                None
            }
        });

        let mut segments = Vec::new();
        if let Some(segs) = item.get("segments").and_then(|v| v.as_array()) {
            for seg in segs {
                let start = match seg.get("start").or_else(|| seg.get("start_timestamp")) {
                    Some(v) => {
                        if let Some(f) = v.as_f64() {
                            f
                        } else if let Some(s) = v.as_str() {
                            s.parse::<f64>().unwrap_or(0.0)
                        } else {
                            0.0
                        }
                    }
                    None => 0.0,
                };
                let end = match seg.get("end").or_else(|| seg.get("end_timestamp")) {
                    Some(v) => {
                        if let Some(f) = v.as_f64() {
                            f
                        } else if let Some(s) = v.as_str() {
                            s.parse::<f64>().unwrap_or(0.0)
                        } else {
                            0.0
                        }
                    }
                    None => 0.0,
                };
                if end > start {
                    segments.push(crate::models::SegmentRange { start, end });
                }
            }
        }

        // Fallback: If AI put start/end directly on the candidate object instead of a segments array
        if segments.is_empty() {
            let start = match item.get("start").or_else(|| item.get("start_timestamp")) {
                Some(v) => {
                    if let Some(f) = v.as_f64() {
                        f
                    } else if let Some(s) = v.as_str() {
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                None => 0.0,
            };
            let end = match item.get("end").or_else(|| item.get("end_timestamp")) {
                Some(v) => {
                    if let Some(f) = v.as_f64() {
                        f
                    } else if let Some(s) = v.as_str() {
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                None => 0.0,
            };
            if end > start {
                segments.push(crate::models::SegmentRange { start, end });
            }
        }

        if !segments.is_empty() {
            drafts.push(CandidateDraft {
                segments,
                score,
                hook,
                rationale,
                title,
                description,
                editing_strategy,
                confidence,
            });
        }
    }

    let mut candidates = drafts
        .into_iter()
        .filter(|candidate| {
            let duration: f64 = candidate.segments.iter().map(|s| s.end - s.start).sum();
            println!(
                "Candidate duration: {}, min_duration: {}",
                duration, min_duration
            );
            duration >= min_duration
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    candidates.truncate(10);
    Ok(candidates)
}

// ─── Spelling Fix ─────────────────────────────────────────────────────────────

const SPELLING_BATCH_SIZE: usize = 80;

/// Fix spelling/punctuation of word texts using the configured LLM.
/// Timestamps are NEVER changed — only the `.text` field of each word is corrected.
pub async fn fix_spelling(
    words: &mut [crate::models::TranscriptWord],
    provider: &str,
    api_key: Option<&str>,
    model_name: Option<&str>,
) -> Result<()> {
    let total = words.len();
    if total == 0 {
        return Ok(());
    }

    for chunk_start in (0..total).step_by(SPELLING_BATCH_SIZE) {
        let chunk_end = (chunk_start + SPELLING_BATCH_SIZE).min(total);
        let word_texts: Vec<&str> = words[chunk_start..chunk_end]
            .iter()
            .map(|w| w.text.as_str())
            .collect();

        let numbered: String = word_texts
            .iter()
            .enumerate()
            .map(|(i, w)| format!("{}: {}", i + 1, w))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a transcription spell-checker. I will give you a numbered list of words from an automatic speech-recognition transcript.\n\
            Your job: fix ONLY spelling and punctuation errors (e.g. 'check gpt' → 'ChatGPT', 'gonna' stays 'gonna', etc.).\n\
            Rules:\n\
            - Do NOT change the order of words.\n\
            - Do NOT remove or add words.\n\
            - Do NOT change filler words or casual speech patterns.\n\
            - Return ONLY the corrected numbered list in the same format. No explanation.\n\n\
            Words:\n{numbered}"
        );

        let corrected_text = match provider {
            "deepseek" => {
                let key = api_key.ok_or_else(|| anyhow!("DeepSeek API key required"))?;
                call_deepseek_raw(&prompt, key).await?
            }
            "claude" => {
                let key = api_key.ok_or_else(|| anyhow!("Claude API key required"))?;
                call_claude_raw(&prompt, key).await?
            }
            "gemini" => {
                let key = api_key.ok_or_else(|| anyhow!("Gemini API key required"))?;
                call_gemini_raw(&prompt, key).await?
            }
            "openai" => {
                let key = api_key.ok_or_else(|| anyhow!("OpenAI API key required"))?;
                call_openai_raw(&prompt, key, "gpt-4o-mini").await?
            }
            "openrouter" => {
                let key = api_key.ok_or_else(|| anyhow!("OpenRouter API key required"))?;
                call_openrouter_raw(
                    &prompt,
                    key,
                    model_name.unwrap_or("meta-llama/llama-3.1-8b-instruct:free"),
                )
                .await?
            }
            "local" => {
                let model = model_name.unwrap_or("qwen2.5");
                call_ollama_raw(&prompt, model).await?
            }
            _ => return Ok(()), // Unknown provider — skip silently
        };

        // Parse the numbered list response and update word texts
        for line in corrected_text.lines() {
            let line = line.trim();
            if let Some(colon_idx) = line.find(": ") {
                let num_str = &line[..colon_idx];
                if let Ok(num) = num_str.trim().parse::<usize>() {
                    let word_idx = chunk_start + num - 1;
                    if word_idx < chunk_end {
                        let corrected = line[colon_idx + 2..].trim().to_string();
                        if !corrected.is_empty() {
                            words[word_idx].text = corrected;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn call_deepseek_raw(prompt: &str, api_key: &str) -> Result<String> {
    let response = get_client()
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
        }))
        .send()
        .await
        .context("calling DeepSeek for spelling fix")?;
    let body: DeepseekResponse = response
        .json()
        .await
        .context("parsing DeepSeek spelling response")?;
    Ok(body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

async fn call_claude_raw(prompt: &str, api_key: &str) -> Result<String> {
    let response = get_client()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": "claude-haiku-20240307",
            "max_tokens": 2048,
            "messages": [{"role": "user", "content": prompt}],
        }))
        .send()
        .await
        .context("calling Claude for spelling fix")?;
    let body: AnthropicMessage = response
        .json()
        .await
        .context("parsing Claude spelling response")?;
    Ok(body
        .content
        .into_iter()
        .find_map(|c| c.text)
        .unwrap_or_default())
}

async fn call_gemini_raw(prompt: &str, api_key: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct GeminiPart {
        text: Option<String>,
    }
    #[derive(Deserialize)]
    struct GeminiContent {
        parts: Vec<GeminiPart>,
    }
    #[derive(Deserialize)]
    struct GeminiCandidate {
        content: GeminiContent,
    }
    #[derive(Deserialize)]
    struct GeminiResponse {
        candidates: Vec<GeminiCandidate>,
    }

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={api_key}");
    let response = get_client()
        .post(&url)
        .json(&json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"temperature": 0.0}
        }))
        .send()
        .await
        .context("calling Gemini for spelling fix")?;
    let body: GeminiResponse = response
        .json()
        .await
        .context("parsing Gemini spelling response")?;
    Ok(body
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .and_then(|p| p.text.clone())
        .unwrap_or_default())
}

async fn call_openai_raw(prompt: &str, api_key: &str, model: &str) -> Result<String> {
    let response = get_client()
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
        }))
        .send()
        .await
        .context("calling OpenAI for spelling fix")?;
    let body: DeepseekResponse = response
        .json()
        .await
        .context("parsing OpenAI spelling response")?;
    Ok(body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

async fn call_openrouter_raw(prompt: &str, api_key: &str, model: &str) -> Result<String> {
    let response = get_client()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.0,
        }))
        .send()
        .await
        .context("calling OpenRouter for spelling fix")?;
    let body: DeepseekResponse = response
        .json()
        .await
        .context("parsing OpenRouter spelling response")?;
    Ok(body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

async fn call_ollama_raw(prompt: &str, model: &str) -> Result<String> {
    let response = get_client()
        .post("http://localhost:11434/api/chat")
        .json(&json!({
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "stream": false,
            "options": {"temperature": 0.0}
        }))
        .send()
        .await
        .context("calling Ollama for spelling fix")?;
    let body: OllamaResponse = response
        .json()
        .await
        .context("parsing Ollama spelling response")?;
    Ok(body.message.content)
}

// ─── Metadata Generation (Title/Caption) ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct CandidateMetadata {
    pub title: String,
    pub description: String,
}

pub async fn generate_candidate_metadata(
    hook: &str,
    rationale: &str,
    transcript_text: &str,
    provider: &str,
    api_key: &str,
    model_name: &str,
) -> Result<CandidateMetadata> {
    let prompt = format!(
        "You are an expert social media manager for TikTok, Instagram Reels, and YouTube Shorts.\n\
        Write a viral title and caption for this short video clip.\n\
        The title should be highly engaging, catchy, and humanized. It MUST include emojis.\n\
        The caption should provide engaging context, use emojis throughout, and include 3-5 relevant hashtags.\n\
        CRITICAL: Do NOT sound like an AI. Use a natural, conversational, human tone.\n\n\
        Clip Hook: {}\n\
        Clip Rationale: {}\n\
        Clip Transcript:\n{}\n\n\
        Return ONLY valid JSON matching this schema:\n\
        {{\"title\":\"...\",\"description\":\"...\"}}",
        hook, rationale, transcript_text
    );

    let raw_response = match provider {
        "deepseek" => call_deepseek_raw(&prompt, api_key).await,
        "claude" => call_claude_raw(&prompt, api_key).await,
        "gemini" => call_gemini_raw(&prompt, api_key).await,
        "openai" => call_openai_raw(&prompt, api_key, model_name).await,
        "openrouter" => call_openrouter_raw(&prompt, api_key, model_name).await,
        "ollama" => call_ollama_raw(&prompt, model_name).await,
        _ => Err(anyhow!("Unsupported LLM provider: {}", provider)),
    }?;

    let json_str = raw_response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(json_str).context("failed to parse metadata JSON")
}

// ─── Senior Editor Review ─────────────────────────────────────────────────────────────

fn build_review_prompt(candidate_text: &str) -> String {
    format!(
        r#"You are an expert YouTube Shorts, Instagram Reels, and TikTok content reviewer.

Your job is NOT to create clips.
Your job is to critically review clips generated by another AI.
Review the clip exactly like a professional video editor whose only goal is to maximize viewer retention and content quality.
Be objective.
Do not try to justify poor decisions.
If something is bad, explain why.

----------------------------------------------------
Review Criteria
----------------------------------------------------

Score each category from 1-10.

1. Story Completeness
2. Hook Strength
3. Context Preservation
4. Flow
5. Segment Quality
6. Pacing
7. Ending
8. Caption Quality (assume 10 unless transcript is extremely messy)
9. Viral Potential
10. Overall Viewer Retention

----------------------------------------------------
Critical Analysis & Improvement Suggestions
----------------------------------------------------
Identify every flaw. For every flaw provide an actionable improvement.

----------------------------------------------------
Final Decision
----------------------------------------------------
Return exactly one decision.
APPROVE
REVISION_REQUIRED
REJECT

Return ONLY valid JSON exactly matching this schema:
{{
  "decision": "APPROVE | REVISION_REQUIRED | REJECT",
  "scores": {{
    "Story_Completeness": 8,
    "Hook_Strength": 7,
    "Context": 9,
    "Flow": 8,
    "Segment_Selection": 8,
    "Pacing": 7,
    "Ending": 8,
    "Caption_Quality": 10,
    "Viral_Potential": 7,
    "Viewer_Retention": 8,
    "Overall_Score": 82,
    "Confidence": 95
  }},
  "flaws": [
    {{
      "issue": "Weak opening",
      "improvement": "Extend Segment 2 by 8 seconds."
    }}
  ]
}}

Transcript to review:
{candidate_text}"#
    )
}

fn parse_review_json(json_str: &str, candidate_id: &str) -> Result<crate::models::CandidateReview> {
    let root: serde_json::Value =
        serde_json::from_str(json_str).context("Failed to parse LLM output as JSON")?;

    let decision = root
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("REVISION_REQUIRED")
        .to_string();

    let scores_obj = root
        .get("scores")
        .ok_or_else(|| anyhow!("Missing 'scores' object"))?;
    let overall_score = scores_obj
        .get("Overall_Score")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let confidence = scores_obj
        .get("Confidence")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let scores = crate::models::ReviewScores {
        story_completeness: scores_obj
            .get("Story_Completeness")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        hook_strength: scores_obj
            .get("Hook_Strength")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        context: scores_obj
            .get("Context")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        flow: scores_obj.get("Flow").and_then(|v| v.as_i64()).unwrap_or(0),
        segment_selection: scores_obj
            .get("Segment_Selection")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        pacing: scores_obj
            .get("Pacing")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        ending: scores_obj
            .get("Ending")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        caption_quality: scores_obj
            .get("Caption_Quality")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        viral_potential: scores_obj
            .get("Viral_Potential")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        viewer_retention: scores_obj
            .get("Viewer_Retention")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    };

    let mut flaws = Vec::new();
    if let Some(flaws_arr) = root.get("flaws").and_then(|v| v.as_array()) {
        for flaw in flaws_arr {
            flaws.push(crate::models::ReviewFlaw {
                issue: flaw
                    .get("issue")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                improvement: flaw
                    .get("improvement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
    }

    Ok(crate::models::CandidateReview {
        id: uuid::Uuid::new_v4().to_string(),
        candidate_id: candidate_id.to_string(),
        decision,
        overall_score,
        confidence,
        scores,
        flaws,
    })
}

pub async fn review_candidate(
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

    let start = raw_response.find('{');
    let end = raw_response.rfind('}');
    let json_str = match (start, end) {
        (Some(s), Some(e)) if s <= e => &raw_response[s..=e],
        _ => &raw_response,
    };

    parse_review_json(json_str, candidate_id)
}

// ─── Viral Prediction ─────────────────────────────────────────────────────────────────

fn build_prediction_prompt(candidate_text: &str) -> String {
    format!(
        r##"You are a world-class YouTube Shorts, Instagram Reels, and TikTok growth strategist.

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
{candidate_text}"##
    )
}

fn parse_prediction_json(
    json_str: &str,
    candidate_id: &str,
) -> Result<crate::models::ViralPrediction> {
    let root: serde_json::Value =
        serde_json::from_str(json_str).context("Failed to parse LLM output as JSON")?;

    let evaluation_summary = root
        .get("evaluation_summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let preds = root
        .get("predictions")
        .ok_or_else(|| anyhow!("Missing 'predictions' object"))?;
    let avg_watch_percentage = preds
        .get("avg_watch_percentage")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let watch_to_end_likelihood = preds
        .get("watch_to_end_likelihood")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let best_target_audience = preds
        .get("best_target_audience")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let best_platform = preds
        .get("best_platform")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let best_posting_time = preds
        .get("best_posting_time")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let best_title = preds
        .get("best_title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let best_thumbnail_text = preds
        .get("best_thumbnail_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut best_hashtags = Vec::new();
    if let Some(tags) = preds.get("best_hashtags").and_then(|v| v.as_array()) {
        for t in tags {
            if let Some(ts) = t.as_str() {
                best_hashtags.push(ts.to_string());
            }
        }
    }

    let expls = root
        .get("explanations")
        .ok_or_else(|| anyhow!("Missing 'explanations' object"))?;
    let viral_reason = expls
        .get("viral_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let failure_reason = expls
        .get("failure_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let single_improvement = expls
        .get("single_improvement")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let scores = root
        .get("scores")
        .ok_or_else(|| anyhow!("Missing 'scores' object"))?;
    let viral_probability = scores
        .get("viral_probability")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let confidence = scores
        .get("confidence")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

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

    let start = raw_response.find('{');
    let end = raw_response.rfind('}');
    let json_str = match (start, end) {
        (Some(s), Some(e)) if s <= e => &raw_response[s..=e],
        _ => &raw_response,
    };

    parse_prediction_json(json_str, candidate_id)
}
