import re

with open('src-tauri/src/models.rs', 'r') as f:
    content = f.read()

models = r'''    pub review: Option<CandidateReview>,
    pub prediction: Option<ViralPrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViralPrediction {
    pub id: String,
    pub candidate_id: String,
    pub evaluation_summary: String,
    pub avg_watch_percentage: String,
    pub watch_to_end_likelihood: String,
    pub best_target_audience: String,
    pub best_platform: String,
    pub best_posting_time: String,
    pub best_title: String,
    pub best_thumbnail_text: String,
    pub best_hashtags: Vec<String>,
    pub viral_reason: String,
    pub failure_reason: String,
    pub single_improvement: String,
    pub viral_probability: i64,
    pub confidence: i64,
}'''
content = re.sub(r'    pub review: Option<CandidateReview>,\n\}', models, content)

with open('src-tauri/src/models.rs', 'w') as f:
    f.write(content)

print("Patched models.rs")
