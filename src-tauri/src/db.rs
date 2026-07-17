use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{
    Candidate, CandidateDraft, Clip, ClipCopy, Project, ProjectDetail, Transcript,
};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path).context("opening SQLite database")?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT,
                source_path TEXT NOT NULL,
                source_duration REAL,
                status TEXT NOT NULL,
                transcription_mode TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS transcripts (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                engine TEXT NOT NULL,
                raw_json TEXT NOT NULL,
                language TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS candidates (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                segments TEXT NOT NULL,
                score REAL NOT NULL,
                hook TEXT NOT NULL,
                rationale TEXT NOT NULL,
                rank INTEGER NOT NULL,
                selected INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS clips (
                id TEXT PRIMARY KEY,
                candidate_id TEXT NOT NULL REFERENCES candidates(id) ON DELETE CASCADE,
                status TEXT NOT NULL,
                output_path TEXT,
                face_track_json TEXT,
                caption_ass_path TEXT,
                render_log TEXT
            );

            CREATE TABLE IF NOT EXISTS clip_copy (
                id TEXT PRIMARY KEY,
                clip_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
                platform TEXT NOT NULL,
                hook_text TEXT,
                caption_text TEXT,
                hashtags TEXT
            );

            CREATE TABLE IF NOT EXISTS schedule_entries (
                id TEXT PRIMARY KEY,
                clip_id TEXT NOT NULL REFERENCES clips(id) ON DELETE CASCADE,
                platform TEXT NOT NULL,
                scheduled_for TEXT,
                status TEXT NOT NULL
            );
            ",
        )?;
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN name TEXT", []);
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN caption_style TEXT", []);
        let _ = conn.execute("ALTER TABLE candidates ADD COLUMN title TEXT", []);
        let _ = conn.execute("ALTER TABLE candidates ADD COLUMN description TEXT", []);
        let _ = conn.execute("ALTER TABLE candidates ADD COLUMN segments TEXT DEFAULT '[]'", []);
        Ok(())
    }

    pub fn create_project(
        &self,
        source_path: &str,
        transcription_mode: &str,
        caption_style: &str,
        source_duration: Option<f64>,
    ) -> Result<Project> {
        let now = Utc::now().to_rfc3339();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: None,
            source_path: source_path.to_string(),
            source_duration,
            status: "ingest".to_string(),
            transcription_mode: transcription_mode.to_string(),
            caption_style: Some(caption_style.to_string()),
            created_at: now.clone(),
            updated_at: now,
        };

        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "INSERT INTO projects (id, name, source_path, source_duration, status, transcription_mode, caption_style, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                project.id,
                project.name,
                project.source_path,
                project.source_duration,
                project.status,
                project.transcription_mode,
                project.caption_style,
                project.created_at,
                project.updated_at
            ],
        )?;

        Ok(project)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, name, source_path, source_duration, status, transcription_mode, created_at, updated_at, caption_style
             FROM projects ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], project_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_project(&self, project_id: &str) -> Result<Project> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.query_row(
            "SELECT id, name, source_path, source_duration, status, transcription_mode, created_at, updated_at, caption_style
             FROM projects WHERE id = ?1",
            params![project_id],
            project_from_row,
        )
        .map_err(Into::into)
    }

    pub fn project_detail(&self, project_id: &str) -> Result<ProjectDetail> {
        let project = self.get_project(project_id)?;
        let transcript = self.latest_transcript(project_id)?;
        let candidates = self.list_candidates(project_id)?;
        let clips = self.list_clips_for_project(project_id)?;
        let copy = self.list_copy_for_project(project_id)?;

        Ok(ProjectDetail {
            project,
            transcript,
            candidates,
            clips,
            copy,
        })
    }

    pub fn update_project_status(
        &self,
        project_id: &str,
        status: &str,
        source_duration: Option<f64>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "UPDATE projects SET status = ?1, source_duration = COALESCE(?2, source_duration), updated_at = ?3 WHERE id = ?4",
            params![status, source_duration, now, project_id],
        )?;
        Ok(())
    }

    pub fn save_transcript(
        &self,
        project_id: &str,
        engine: &str,
        raw_json: &str,
        language: Option<&str>,
    ) -> Result<Transcript> {
        let transcript = Transcript {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            engine: engine.to_string(),
            raw_json: raw_json.to_string(),
            language: language.map(ToOwned::to_owned),
            created_at: Utc::now().to_rfc3339(),
        };

        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "DELETE FROM transcripts WHERE project_id = ?1",
            params![project_id],
        )?;
        conn.execute(
            "INSERT INTO transcripts (id, project_id, engine, raw_json, language, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                transcript.id,
                transcript.project_id,
                transcript.engine,
                transcript.raw_json,
                transcript.language,
                transcript.created_at
            ],
        )?;
        Ok(transcript)
    }

    pub fn latest_transcript(&self, project_id: &str) -> Result<Option<Transcript>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.query_row(
            "SELECT id, project_id, engine, raw_json, language, created_at
             FROM transcripts WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
            params![project_id],
            |row| {
                Ok(Transcript {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    engine: row.get(2)?,
                    raw_json: row.get(3)?,
                    language: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn add_candidates(
        &self,
        project_id: &str,
        drafts: &[CandidateDraft],
    ) -> Result<Vec<Candidate>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        
        let max_rank: i64 = conn.query_row(
            "SELECT COALESCE(MAX(rank), 0) FROM candidates WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        ).unwrap_or(0);


        let selected_cutoff = drafts.len().min(6).max(3).min(drafts.len());
        let mut candidates = Vec::with_capacity(drafts.len());

        for (index, draft) in drafts.iter().enumerate() {
            let candidate = Candidate {
                id: Uuid::new_v4().to_string(),
                project_id: project_id.to_string(),
                segments: draft.segments.clone(),
                score: draft.score,
                hook: draft.hook.clone(),
                rationale: draft.rationale.clone(),
                rank: max_rank + (index as i64) + 1,
                selected: index < selected_cutoff,
                title: draft.title.clone(),
                description: draft.description.clone(), review: None, prediction: None,
            };

            let segments_json = serde_json::to_string(&candidate.segments).unwrap_or_else(|_| "[]".to_string());

            conn.execute(
                "INSERT INTO candidates (id, project_id, segments, score, hook, rationale, rank, selected, title, description, start_sec, end_sec)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0.0, 0.0)",
                params![
                    &candidate.id,
                    &candidate.project_id,
                    segments_json,
                    candidate.score,
                    &candidate.hook,
                    &candidate.rationale,
                    candidate.rank,
                    if candidate.selected { 1 } else { 0 },
                    &candidate.title,
                    &candidate.description,
                ],
            )?;

            conn.execute(
                "INSERT INTO clips (id, candidate_id, status) VALUES (?1, ?2, 'pending')",
                params![Uuid::new_v4().to_string(), &candidate.id],
            )?;

            candidates.push(candidate);
        }

        Ok(candidates)
    }

    pub fn add_manual_candidate(
        &self,
        project_id: &str,
        start_sec: f64,
        end_sec: f64,
    ) -> Result<Candidate> {
        let conn = self.conn.lock().expect("database mutex poisoned");

        // Find the maximum rank for this project
        let max_rank: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(rank), 0) FROM candidates WHERE project_id = ?1",
                params![project_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let candidate = Candidate {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            segments: vec![crate::models::SegmentRange { start: start_sec, end: end_sec }],
            score: 1.0, // perfect score for user's manual clip
            hook: "Custom Manual Clip".to_string(),
            rationale: "Manually added by user.".to_string(),
            rank: max_rank + 1,
            selected: true,
            title: None,
            description: None, review: None, prediction: None,
        };

        let segments_json = serde_json::to_string(&candidate.segments).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO candidates (id, project_id, segments, score, hook, rationale, rank, selected, title, description, start_sec, end_sec)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0.0, 0.0)",
            params![
                &candidate.id,
                &candidate.project_id,
                segments_json,
                candidate.score,
                &candidate.hook,
                &candidate.rationale,
                candidate.rank,
                if candidate.selected { 1 } else { 0 },
                &candidate.title,
                &candidate.description
            ],
        )?;

        conn.execute(
            "INSERT INTO clips (id, candidate_id, status) VALUES (?1, ?2, 'pending')",
            params![Uuid::new_v4().to_string(), &candidate.id],
        )?;

        Ok(candidate)
    }

    pub fn list_candidates(&self, project_id: &str) -> Result<Vec<Candidate>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, project_id, segments, score, hook, rationale, rank, selected, title, description
             FROM candidates WHERE project_id = ?1 ORDER BY rank ASC",
        )?;
        let rows = stmt.query_map(params![project_id], candidate_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn get_candidate_with_project(&self, candidate_id: &str) -> Result<(Candidate, Project)> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.query_row(
            "SELECT
                candidates.id, candidates.project_id, candidates.segments,
                candidates.score, candidates.hook, candidates.rationale, candidates.rank, candidates.selected,
                projects.id, projects.name, projects.source_path, projects.source_duration, projects.status,
                projects.transcription_mode, projects.created_at, projects.updated_at, projects.caption_style,
                candidates.title, candidates.description
             FROM candidates
             INNER JOIN projects ON projects.id = candidates.project_id
             WHERE candidates.id = ?1",
            params![candidate_id],
            |row| {
                let selected: i64 = row.get(7)?;
                let segments_str: String = row.get(2)?;
                let segments = serde_json::from_str(&segments_str).unwrap_or_default();
                let candidate = Candidate {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    segments,
                    score: row.get(3)?,
                    hook: row.get(4)?,
                    rationale: row.get(5)?,
                    rank: row.get(6)?,
                    selected: selected == 1,
                    title: row.get(17)?,
                    description: row.get(18)?,
                    review: None, prediction: None,
                };
                let project = Project {
                    id: row.get(8)?,
                    name: row.get(9)?,
                    source_path: row.get(10)?,
                    source_duration: row.get(11)?,
                    status: row.get(12)?,
                    transcription_mode: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                    caption_style: row.get(16)?,
                };
                Ok((candidate, project))
            },
        )
        .map_err(Into::into)
    }

    pub fn update_clip_for_candidate(
        &self,
        candidate_id: &str,
        status: &str,
        output_path: Option<&str>,
        caption_ass_path: Option<&str>,
        render_log: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "UPDATE clips
             SET status = ?1,
                 output_path = COALESCE(?2, output_path),
                 caption_ass_path = COALESCE(?3, caption_ass_path),
                 render_log = COALESCE(?4, render_log)
             WHERE candidate_id = ?5",
            params![status, output_path, caption_ass_path, render_log, candidate_id],
        )?;
        Ok(())
    }

    pub fn set_selected_clip_count(
        &self,
        project_id: &str,
        count: usize,
    ) -> Result<Vec<Candidate>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "UPDATE candidates SET selected = CASE WHEN rank <= ?1 THEN 1 ELSE 0 END WHERE project_id = ?2",
            params![count as i64, project_id],
        )?;
        drop(conn);
        self.list_candidates(project_id)
    }

    pub fn update_candidate_metadata(
        &self,
        candidate_id: &str,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            "UPDATE candidates SET title = COALESCE(?1, title), description = COALESCE(?2, description) WHERE id = ?3",
            params![title, description, candidate_id],
        )?;
        Ok(())
    }

    fn list_clips_for_project(&self, project_id: &str) -> Result<Vec<Clip>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT clips.id, clips.candidate_id, clips.status, clips.output_path, clips.face_track_json, clips.caption_ass_path, clips.render_log
             FROM clips
             INNER JOIN candidates ON candidates.id = clips.candidate_id
             WHERE candidates.project_id = ?1
             ORDER BY candidates.rank ASC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(Clip {
                id: row.get(0)?,
                candidate_id: row.get(1)?,
                status: row.get(2)?,
                output_path: row.get(3)?,
                face_track_json: row.get(4)?,
                caption_ass_path: row.get(5)?,
                render_log: row.get(6)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn list_copy_for_project(&self, project_id: &str) -> Result<Vec<ClipCopy>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT clip_copy.id, clip_copy.clip_id, clip_copy.platform, clip_copy.hook_text, clip_copy.caption_text, clip_copy.hashtags
             FROM clip_copy
             INNER JOIN clips ON clips.id = clip_copy.clip_id
             INNER JOIN candidates ON candidates.id = clips.candidate_id
             WHERE candidates.project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(ClipCopy {
                id: row.get(0)?,
                clip_id: row.get(1)?,
                platform: row.get(2)?,
                hook_text: row.get(3)?,
                caption_text: row.get(4)?,
                hashtags: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn delete_project(&self, project_id: &str) -> Result<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
        Ok(())
    }

    pub fn rename_project(&self, project_id: &str, name: &str) -> Result<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, project_id],
        )?;
        Ok(())
    }

    pub fn save_candidate_review(&self, review: &crate::models::CandidateReview) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?;
        
        let scores_json = serde_json::to_string(&review.scores).unwrap_or_else(|_| "{}".to_string());
        let flaws_json = serde_json::to_string(&review.flaws).unwrap_or_else(|_| "[]".to_string());
        
        conn.execute(
            "INSERT INTO candidate_reviews (id, candidate_id, decision, overall_score, confidence, scores_json, flaws_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(candidate_id) DO UPDATE SET
                decision = excluded.decision,
                overall_score = excluded.overall_score,
                confidence = excluded.confidence,
                scores_json = excluded.scores_json,
                flaws_json = excluded.flaws_json",
            rusqlite::params![
                &review.id,
                &review.candidate_id,
                &review.decision,
                review.overall_score,
                review.confidence,
                scores_json,
                flaws_json
            ],
        )?;
        Ok(())
    }

    pub fn save_candidate_prediction(&self, prediction: &crate::models::ViralPrediction) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("DB lock poisoned: {}", e))?;
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
}

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        source_path: row.get(2)?,
        source_duration: row.get(3)?,
        status: row.get(4)?,
        transcription_mode: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        caption_style: row.get(8)?,
    })
}


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
            scores: serde_json::from_str(&scores_json).unwrap_or(crate::models::ReviewScores {
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
