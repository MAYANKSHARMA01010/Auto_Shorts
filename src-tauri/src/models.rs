use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub data_dir: String,
    pub has_ffmpeg: bool,
    pub has_ffprobe: bool,
    pub has_deepgram_key: bool,
    pub has_anthropic_key: bool,
    pub has_deepseek_key: bool,
    pub has_gemini_key: bool,
    pub has_openai_key: bool,
    pub has_openrouter_key: bool,
    pub llm_provider: String,
    pub has_local_whisper_model: bool,
    pub has_ollama: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaProbe {
    pub duration_sec: Option<f64>,
    pub has_video: bool,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: Option<String>,
    pub source_path: String,
    pub source_duration: Option<f64>,
    pub status: String,
    pub transcription_mode: String,
    pub caption_style: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub id: String,
    pub project_id: String,
    pub engine: String,
    pub raw_json: String,
    pub language: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub project_id: String,
    pub segments: Vec<SegmentRange>,
    pub score: f64,
    pub hook: String,
    pub rationale: String,
    pub rank: i64,
    pub selected: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub editing_strategy: Option<String>,
    pub confidence: Option<f64>,
    pub review: Option<CandidateReview>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewScores {
    pub story_completeness: i64,
    pub hook_strength: i64,
    pub context: i64,
    pub flow: i64,
    pub segment_selection: i64,
    pub pacing: i64,
    pub ending: i64,
    pub caption_quality: i64,
    pub viral_potential: i64,
    pub viewer_retention: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFlaw {
    pub issue: String,
    pub improvement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReview {
    pub id: String,
    pub candidate_id: String,
    pub decision: String,
    pub overall_score: i64,
    pub confidence: i64,
    pub scores: ReviewScores,
    pub flaws: Vec<ReviewFlaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub candidate_id: String,
    pub status: String,
    pub output_path: Option<String>,
    pub face_track_json: Option<String>,
    pub caption_ass_path: Option<String>,
    pub render_log: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipCopy {
    pub id: String,
    pub clip_id: String,
    pub platform: String,
    pub hook_text: Option<String>,
    pub caption_text: Option<String>,
    pub hashtags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: Project,
    pub transcript: Option<Transcript>,
    pub candidates: Vec<Candidate>,
    pub clips: Vec<Clip>,
    pub copy: Vec<ClipCopy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedTranscript {
    pub language: String,
    pub duration: f64,
    pub speakers: Vec<String>,
    pub words: Vec<TranscriptWord>,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptWord {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub speaker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub start: f64,
    pub end: f64,
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateDraft {
    pub segments: Vec<SegmentRange>,
    pub score: f64,
    pub hook: String,
    pub rationale: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub editing_strategy: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentRange {
    pub start: f64,
    pub end: f64,
}
