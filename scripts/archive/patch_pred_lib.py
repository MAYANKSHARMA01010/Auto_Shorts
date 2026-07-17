import re

with open('src-tauri/src/lib.rs', 'r') as f:
    content = f.read()

new_cmd = r'''
#[tauri::command]
async fn predict_viral_cmd(
    state: tauri::State<'_, AppState>,
    candidate_id: String,
    provider: String,
    api_key: Option<String>,
    model_name: Option<String>,
) -> Result<crate::models::ViralPrediction, String> {
    // We need the transcript to pull out the candidate's text.
    let (candidate, project) = state.db.get_candidate_with_project(&candidate_id).map_err(|e| e.to_string())?;
    
    // Get the global transcript
    let transcript = state.db.latest_transcript(&project.id).map_err(|e| e.to_string())?;
    let transcript = transcript.ok_or_else(|| "No transcript found".to_string())?;
    let normalized_transcript: NormalizedTranscript = serde_json::from_str(&transcript.raw_json)
        .map_err(|e| e.to_string())?;
        
    // Extract the text for the candidate's segments
    let mut candidate_text = String::new();
    let remapped_words = crate::remap_transcript(&normalized_transcript.words, &candidate.segments);
    for w in remapped_words {
        candidate_text.push_str(&w.text);
        candidate_text.push(' ');
    }
    
    // Call the LLM
    let prediction = llm::predict_viral(
        &candidate_id,
        &candidate_text.trim(),
        &provider,
        api_key.as_deref(),
        model_name.as_deref(),
    ).await.map_err(|e| e.to_string())?;
    
    // Save the prediction
    state.db.save_candidate_prediction(&prediction).map_err(|e| e.to_string())?;
    
    Ok(prediction)
}
'''
# Insert before generate_candidates
content = re.sub(r'(#\[tauri::command\]\nasync fn generate_candidates\()', new_cmd + r'\n\1', content)

# Add to invoke_handler
content = re.sub(r'review_candidate_cmd,', r'review_candidate_cmd,\n            predict_viral_cmd,', content)

with open('src-tauri/src/lib.rs', 'w') as f:
    f.write(content)

print("Patched lib.rs for prediction")
