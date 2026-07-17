# Tauri API Commands

The frontend communicates with the backend exclusively via Tauri commands. The following commands are registered and exposed to the Next.js frontend.

## Project Management

### `create_project`
Creates a new project entry in the SQLite database and initializes the workspace directory.
**Args:** `video_path: String`
**Returns:** `Project` object.

### `get_projects`
Retrieves a paginated list of all projects.
**Returns:** `Vec<Project>`

### `get_project`
Retrieves a single project by ID.
**Args:** `project_id: String`
**Returns:** `Project` object.

### `delete_project`
Deletes a project and all its associated disk data.
**Args:** `project_id: String`

## Processing Pipeline

### `transcribe_project`
Initiates audio extraction and sends the audio to the selected transcription provider (Deepgram or Whisper).
**Args:** `project_id: String`, `provider: String`, `llm_model_name: Option<String>`
**Returns:** `Transcript` object.

### `generate_clips`
Triggers the LLM to process the transcript and identify viral short-form segments.
**Args:** `project_id: String`, `provider: String`, `model_name: String`
**Returns:** `Vec<ClipDraft>`

### `render_clip`
Instructs FFmpeg to stitch segments together, burn VTT captions, and output the final video file.
**Args:** `project_id: String`, `draft_id: String`
**Returns:** `RenderJobStatus`

## Utilities

### `get_env_keys`
Reads configured API keys from the local `.env` securely.
**Returns:** JSON object containing loaded key names.
