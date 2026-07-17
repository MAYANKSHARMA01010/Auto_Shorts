# AutoShorts 🚀

> **Local-first long-form to short-form clip studio.**

AutoShorts is a desktop application that automatically processes long-form videos (like podcasts and streams) and turns them into highly engaging vertical short-form clips (for YouTube Shorts, TikTok, Instagram Reels) using AI.

![Screenshot Placeholder](https://via.placeholder.com/800x400?text=AutoShorts+Dashboard)

## Features

- **Automated Clipping:** Upload a video and instantly extract the best 30-90 second segments.
- **Multi-Provider AI:** Supports OpenAI, Claude, DeepSeek, Gemini, OpenRouter, and local Ollama to find clips.
- **Smart Captions:** Burns perfectly timed `.vtt` subtitles directly into the video.
- **Local Rendering:** Uses FFmpeg to perform blazing-fast, privacy-preserving local video editing.
- **Viral Prediction Engine:** Evaluates generated clips based on hook strength, retention probability, and viral score.

## Architecture

AutoShorts is built using a modern **Tauri** architecture.
- **Frontend:** Next.js (React 19) for a snappy, responsive UI.
- **Backend:** Rust for memory-safe and highly concurrent native operations.
- **Database:** SQLite (embedded via `rusqlite`) for project, transcript, and clip persistence.
- **Media Pipeline:** FFmpeg/FFprobe for video manipulation and probe extraction.

*See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for deeper technical details.*

## Installation & Setup

1. **Prerequisites:**
   - [Node.js (v18+)](https://nodejs.org/) & [pnpm](https://pnpm.io/)
   - [Rust & Cargo](https://rustup.rs/)
   - [FFmpeg & FFprobe](https://ffmpeg.org/download.html) (Ensure they are on your system PATH).

2. **Clone & Install:**
   ```bash
   git clone https://github.com/your-username/autoshorts.git
   cd autoshorts
   pnpm install
   ```

3. **Development Mode:**
   ```bash
   pnpm run tauri:dev
   ```

4. **Production Build:**
   ```bash
   pnpm run build
   pnpm run tauri:build
   ```

## Environment Variables

AutoShorts utilizes standard `.env` configuration for API keys.
*See [docs/ENV_KEYS_GUIDE.md](docs/ENV_KEYS_GUIDE.md) for how to configure OpenAI, Deepgram, and others.*

## Project Structure

```text
├── docs/                # Project documentation
├── frontend/            # Next.js React Application
├── scripts/             # Internal orchestration & patch scripts
└── src-tauri/           # Rust Tauri Backend
    ├── src/             
    │   ├── db.rs        # SQLite schema and logic
    │   ├── llm.rs       # Multi-provider LLM integrations
    │   ├── media.rs     # FFmpeg rendering engine
    │   └── transcription.rs # Deepgram audio transcription
```

## Troubleshooting

Running into issues with missing FFmpeg, dead API keys, or broken `.vtt` parsers? 
*See [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md).*

## License

MIT License. See `LICENSE` for more information.
