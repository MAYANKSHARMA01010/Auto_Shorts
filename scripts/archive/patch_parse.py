import re

with open('src-tauri/src/llm.rs', 'r') as f:
    content = f.read()

new_parse = """fn parse_candidate_json(text: &str, _min_duration: f64) -> Result<Vec<CandidateDraft>> {
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
        let title = item.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
        
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

        let mut segments = Vec::new();
        if let Some(segs) = item.get("segments").and_then(|v| v.as_array()) {
            for seg in segs {
                let start = match seg.get("start").or_else(|| seg.get("start_timestamp")) {
                    Some(v) => {
                        if let Some(f) = v.as_f64() { f }
                        else if let Some(s) = v.as_str() { s.parse::<f64>().unwrap_or(0.0) }
                        else { 0.0 }
                    }
                    None => 0.0,
                };
                let end = match seg.get("end").or_else(|| seg.get("end_timestamp")) {
                    Some(v) => {
                        if let Some(f) = v.as_f64() { f }
                        else if let Some(s) = v.as_str() { s.parse::<f64>().unwrap_or(0.0) }
                        else { 0.0 }
                    }
                    None => 0.0,
                };
                if end > start {
                    segments.push(crate::models::SegmentRange { start, end });
                }
            }
        }

        if !segments.is_empty() {
            drafts.push(CandidateDraft {
                segments,
                score,
                hook,
                rationale,
                title,
            });
        }
    }

    let mut candidates = drafts
        .into_iter()
        .filter(|candidate| {
            let duration: f64 = candidate.segments.iter().map(|s| s.end - s.start).sum();
            duration >= 30.0 && duration <= 180.0
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    candidates.truncate(10);
    Ok(candidates)
}"""

old_parse_regex = r'fn parse_candidate_json\(text: &str, _min_duration: f64\) -> Result<Vec<CandidateDraft>> \{[\s\S]*?Ok\(candidates\)\n\}'
content = re.sub(old_parse_regex, new_parse, content)

with open('src-tauri/src/llm.rs', 'w') as f:
    f.write(content)

print("Patched llm.rs parsing")
