# Architecture

AutoShorts follows a hybrid architecture, combining a Next.js (React) frontend with a native Rust backend powered by Tauri.

## 1. Tauri Backend (Rust)
The core logic resides in `src-tauri/src`. It handles all heavy computational, IO-bound, and systemic tasks.

- **`main.rs`:** Initializes the Tauri application, registers commands, and sets up state (database connection pool, config paths).
- **`db.rs`:** Manages SQLite database state. It relies on the `rusqlite` crate. Features schema migrations and thread-safe locking using `std::sync::Mutex`.
- **`transcription.rs`:** Manages the integration with the Deepgram API for generating transcripts and word-level timestamps, as well as fallback local transcription (Whisper).
- **`llm.rs`:** Defines the trait-based architecture for communicating with various AI providers (OpenAI, Claude, DeepSeek, Gemini, OpenRouter, Ollama). It formats the transcript data, manages rate limits, and validates the returned JSON.
- **`media.rs`:** The multimedia powerhouse. It wraps FFmpeg and FFprobe binaries. It executes commands to crop, burn subtitles, apply watermarks, and extract high-quality audio representations.
- **`models.rs`:** Common structured types for serialization and deserialization across the application boundary.

## 2. Frontend (Next.js)
The frontend in `frontend/` acts entirely as a presentation layer. 

- **Static Export:** The Next.js application is statically compiled (`out/`) and bundled directly into the Tauri binary, giving it the feel of a native application.
- **IPC (Inter-Process Communication):** React components communicate with the Rust backend using `@tauri-apps/api`. No backend REST server is spun up; instead, the application leverages direct message passing across the Tauri bridge.

## 3. Data Flow
1. **Upload:** User selects a video. Tauri invokes `ffmpeg` to probe it and save metadata to SQLite.
2. **Transcription:** Audio is extracted and sent to Deepgram (or Whisper locally) by `transcription.rs`. Timestamps are cached.
3. **Clipping:** `llm.rs` sends the full transcript to an AI model with a specialized prompt to extract the most viral segments.
4. **Rendering:** `media.rs` sequences the extracted clips, burns `.vtt` subtitles onto them, and writes the final `.mp4` payload to disk.
