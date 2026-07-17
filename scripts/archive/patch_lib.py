import re

with open('src-tauri/src/lib.rs', 'r') as f:
    content = f.read()

new_cmd = r'''
#[tauri::command]
async fn review_candidate_cmd(
    state: tauri::State<'_, AppState>,
    candidate_id: String,
    provider: String,
    api_key: Option<String>,
    model_name: Option<String>,
) -> Result<crate::models::CandidateReview, String> {
    // We need the transcript to pull out the candidate's text.
    let candidate = state.db.get_candidate_with_project(&candidate_id).map_err(|e| e.to_string())?;
    
    // Get the global transcript
    let transcript = state.db.get_transcript(&candidate.project.id).map_err(|e| e.to_string())?;
    let mut normalized_transcript: NormalizedTranscript = serde_json::from_str(&transcript.raw_json)
        .map_err(|e| e.to_string())?;
        
    // Extract the text for the candidate's segments
    let mut candidate_text = String::new();
    for seg in &candidate.segments {
        let (words, _) = crate::media::remap_transcript(&mut normalized_transcript, seg.start, seg.end);
        for w in words {
            candidate_text.push_str(&w.text);
            candidate_text.push(' ');
        }
    }
    
    // Call the LLM
    let review = llm::review_candidate(
        &candidate_id,
        &candidate_text.trim(),
        &provider,
        api_key.as_deref(),
        model_name.as_deref(),
    ).await.map_err(|e| e.to_string())?;
    
    // Save the review
    state.db.save_candidate_review(&review).map_err(|e| e.to_string())?;
    
    Ok(review)
}
'''
# Insert before generate_candidates
content = re.sub(r'(#\[tauri::command\]\nasync fn generate_candidates\()', new_cmd + r'\n\1', content)

# Add to invoke_handler
content = re.sub(r'generate_candidate_metadata_cmd,', r'generate_candidate_metadata_cmd,\n            review_candidate_cmd,', content)

with open('src-tauri/src/lib.rs', 'w') as f:
    f.write(content)

print("Patched lib.rs")
