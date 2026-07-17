import re

with open('src-tauri/src/db.rs', 'r') as f:
    content = f.read()

# 1. Add schema migration
migration_old = r'''        "CREATE TABLE IF NOT EXISTS candidate_reviews \(
            id TEXT PRIMARY KEY,
            candidate_id TEXT NOT NULL UNIQUE,
            decision TEXT NOT NULL,
            overall_score INTEGER NOT NULL,
            confidence INTEGER NOT NULL,
            scores_json TEXT NOT NULL,
            flaws_json TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY\(candidate_id\) REFERENCES candidates\(id\)
        \)",'''
migration_new = migration_old + r'''
        "CREATE TABLE IF NOT EXISTS candidate_predictions (
            id TEXT PRIMARY KEY,
            candidate_id TEXT NOT NULL UNIQUE,
            data_json TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(candidate_id) REFERENCES candidates(id)
        )",'''
content = re.sub(migration_old, migration_new, content)

# 2. Add prediction: None to Candidate instantiations
# I'll find `review: None,` and replace it with `review: None, prediction: None,`
content = re.sub(r'review: None,', r'review: None, prediction: None,', content)

# 3. Modify list_candidates query
list_old = r'''        let mut stmt = conn.prepare\(
            "SELECT c.id, c.project_id, c.segments, c.score, c.hook, c.rationale, c.rank, c.selected, c.title, c.description,
                    r.id as review_id, r.candidate_id as review_candidate_id, r.decision as review_decision, 
                    r.overall_score as review_overall_score, r.confidence as review_confidence, r.scores_json as review_scores, r.flaws_json as review_flaws
             FROM candidates c 
             LEFT JOIN candidate_reviews r ON c.id = r.candidate_id
             WHERE c.project_id = \?1 ORDER BY c.rank ASC"
        \)\?;'''
list_new = r'''        let mut stmt = conn.prepare(
            "SELECT c.id, c.project_id, c.segments, c.score, c.hook, c.rationale, c.rank, c.selected, c.title, c.description,
                    r.id as review_id, r.candidate_id as review_candidate_id, r.decision as review_decision, 
                    r.overall_score as review_overall_score, r.confidence as review_confidence, r.scores_json as review_scores, r.flaws_json as review_flaws,
                    p.id as pred_id, p.data_json as pred_data_json
             FROM candidates c 
             LEFT JOIN candidate_reviews r ON c.id = r.candidate_id
             LEFT JOIN candidate_predictions p ON c.id = p.candidate_id
             WHERE c.project_id = ?1 ORDER BY c.rank ASC"
        )?;'''
content = re.sub(list_old, list_new, content)

# 4. Modify get_candidate_with_project query
get_cand_old = r'''        let candidate = conn.query_row\(
            "SELECT c.id, c.project_id, c.segments, c.score, c.hook, c.rationale, c.rank, c.selected, c.title, c.description,
                    r.id as review_id, r.candidate_id as review_candidate_id, r.decision as review_decision, 
                    r.overall_score as review_overall_score, r.confidence as review_confidence, r.scores_json as review_scores, r.flaws_json as review_flaws
             FROM candidates c 
             LEFT JOIN candidate_reviews r ON c.id = r.candidate_id
             WHERE c.id = \?1",'''
get_cand_new = r'''        let candidate = conn.query_row(
            "SELECT c.id, c.project_id, c.segments, c.score, c.hook, c.rationale, c.rank, c.selected, c.title, c.description,
                    r.id as review_id, r.candidate_id as review_candidate_id, r.decision as review_decision, 
                    r.overall_score as review_overall_score, r.confidence as review_confidence, r.scores_json as review_scores, r.flaws_json as review_flaws,
                    p.id as pred_id, p.data_json as pred_data_json
             FROM candidates c 
             LEFT JOIN candidate_reviews r ON c.id = r.candidate_id
             LEFT JOIN candidate_predictions p ON c.id = p.candidate_id
             WHERE c.id = ?1",'''
content = re.sub(get_cand_old, get_cand_new, content)

# 5. Modify candidate_from_row to parse prediction
new_candidate_from_row = r'''
fn candidate_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Candidate> {
    let selected: i64 = row.get(7)?;
    let segments_str: String = row.get(2)?;
    let segments = serde_json::from_str(&segments_str).unwrap_or_default();
    
    // Review parsing
    let review_id: Option<String> = row.get("review_id").unwrap_or_default();
    let review = if let Some(id) = review_id {
        let scores_json: String = row.get("review_scores").unwrap_or_else(|_| "{}".to_string());
        let flaws_json: String = row.get("review_flaws").unwrap_or_else(|_| "[]".to_string());
        Some(crate::models::CandidateReview {
            id,
            candidate_id: row.get("review_candidate_id").unwrap_or_default(),
            decision: row.get("review_decision").unwrap_or_default(),
            overall_score: row.get("review_overall_score").unwrap_or(0),
            confidence: row.get("review_confidence").unwrap_or(0),
            scores: serde_json::from_str(&scores_json).unwrap_or_else(|_| crate::models::ReviewScores {
                story_completeness: 0, hook_strength: 0, context: 0, flow: 0, segment_selection: 0, pacing: 0, ending: 0, caption_quality: 0, viral_potential: 0, viewer_retention: 0
            }),
            flaws: serde_json::from_str(&flaws_json).unwrap_or_default(),
        })
    } else {
        None
    };

    // Prediction parsing
    let pred_id: Option<String> = row.get("pred_id").unwrap_or_default();
    let prediction = if pred_id.is_some() {
        let data_json: String = row.get("pred_data_json").unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&data_json).ok()
    } else {
        None
    };

    Ok(Candidate {
        id: row.get(0)?,
        project_id: row.get(1)?,
        segments,
        score: row.get(3)?,
        hook: row.get(4)?,
        rationale: row.get(5)?,
        rank: row.get(6)?,
        selected: selected == 1,
        title: row.get(8)?,
        description: row.get(9)?,
        review,
        prediction,
    })
}
'''
old_candidate_from_row = r'fn candidate_from_row\(row: &rusqlite::Row<\'_>\) -> rusqlite::Result<Candidate> \{[\s\S]*?\}\n'
content = re.sub(old_candidate_from_row, new_candidate_from_row, content)

# 6. Add save_candidate_prediction
save_pred = r'''
    pub fn save_candidate_prediction(&self, prediction: &crate::models::ViralPrediction) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let data_json = serde_json::to_string(prediction).unwrap_or_else(|_| "{}".to_string());
        
        conn.execute(
            "INSERT INTO candidate_predictions (id, candidate_id, data_json)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(candidate_id) DO UPDATE SET
                data_json = excluded.data_json",
            rusqlite::params![
                &prediction.id,
                &prediction.candidate_id,
                data_json
            ],
        )?;
        Ok(())
    }
'''
content = re.sub(r'(\n\}[\s]*)$', save_pred + r'\1', content)

with open('src-tauri/src/db.rs', 'w') as f:
    f.write(content)

print("Patched db.rs")
