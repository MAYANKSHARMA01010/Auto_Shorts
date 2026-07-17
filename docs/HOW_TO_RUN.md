# AutoShorts — How to Run

## The Only 2 Commands You Need

Run both from the **root `autoshorts/` folder** — nowhere else.

```bash
# Use the app every day
pnpm run tauri:dev
```

```bash
# Package the app as a .app file to share or distribute
pnpm run tauri:build
```

That's it. Nothing else is needed.

---

## How It Works (Simple Explanation)

AutoShorts is a **native Mac Desktop App**, not a website.

When you run `pnpm run tauri:dev`:
- It starts the Next.js UI on port 3000 (internally — don't open this in your browser)
- It compiles and starts the Rust backend (no port — embedded inside the app)
- It opens a native Desktop window where everything works together

The frontend (UI) talks to the backend (Rust) through a special Tauri bridge called `invoke()`. This only works inside the Desktop window — not in Chrome or Safari.

> **Never open `localhost:3000` in a browser.** Every button will crash with `TypeError: Cannot read properties of undefined (reading 'invoke')`.

---

## What the Other Scripts Do (Ignore These)

| Script | Purpose | Should you use it? |
|---|---|---|
| `pnpm run tauri:dev` | ✅ Start the full app | **Yes — your main command** |
| `pnpm run tauri:build` | ✅ Package as `.app` | **Yes — only when distributing** |
| `pnpm run dev` | UI design only (Rust not running, buttons crash) | No |
| `pnpm run build` | Builds Next.js UI only | No |
| `pnpm run tauri` | Raw Tauri CLI | No |

The `dev` and `build` scripts exist only for developers doing UI-only design work. They are not for running the full app.

---

## First Time Setup (Do Once)

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Restart terminal after this
```

### 2. Install Node.js + pnpm
```bash
brew install node
npm install -g pnpm
```

### 3. Install FFmpeg with caption support
```bash
brew install ffmpeg-full
echo 'export PATH="/opt/homebrew/opt/ffmpeg-full/bin:$PATH"' >> ~/.zshrc
```
> Use `ffmpeg-full`, not plain `ffmpeg`. The standard `ffmpeg` is missing `libfreetype` and captions will be skipped silently.

### 4. Install Whisper (for offline transcription)
```bash
pip3 install openai-whisper --break-system-packages
```

### 5. Install dependencies
```bash
# From the root autoshorts/ folder
pnpm install
```

### 6. Set up `.env`
Open `.env` in the root folder. Every key must be in `KEY=value` format:
```env
DEEPGRAM_API_KEY=your_key_here
ANTHROPIC_API_KEY=your_key_here
DEEPSEEK_API_KEY=your_key_here
GEMINI_API_KEY=your_key_here
OPENAI_API_KEY=your_key_here
OPENROUTER_API_KEY=your_key_here
LLM_PROVIDER=deepseek
```
You only need to fill in the keys for the providers you want to use. At minimum, fill in one LLM key.

---

## How to See Logs When Something Goes Wrong

| What to check | How |
|---|---|
| Rust backend errors | Watch the terminal running `tauri:dev` |
| Frontend JS errors | Press `Cmd + Option + I` inside the Desktop window |

---

## Troubleshooting

**`Cannot read properties of undefined (reading 'invoke')`**  
You opened `localhost:3000` in a browser. Use the Desktop window instead.

**`failed to bundle project: error running bundle_dmg.sh`**  
The DMG packaging step is disabled — `tauri:build` now only creates the `.app` file. If you still see this, check `src-tauri/tauri.conf.json` and ensure `"targets": ["app"]`.

**`No such filter: 'drawtext'` / captions skipped**  
You have standard `ffmpeg` instead of `ffmpeg-full`. Fix:
```bash
brew install ffmpeg-full
echo 'export PATH="/opt/homebrew/opt/ffmpeg-full/bin:$PATH"' >> ~/.zshrc
# Open a new terminal, then restart tauri:dev
```

**`EADDRINUSE: address already in use :::3000`**  
Port 3000 is already taken (you ran `pnpm run dev` in `frontend/` manually). Kill it:
```bash
lsof -ti:3000 | xargs kill -9
```
Then run `pnpm run tauri:dev` again.

**`ENOENT: no such file or directory, pages-manifest.json`**  
Two builds ran at the same time and corrupted the cache. Fix:
```bash
rm -rf frontend/.next
pnpm run tauri:dev
```

**API key not detected**  
Make sure the key is not just a raw value. It must have a variable name:
```env
# Wrong — this is just a comment, ignored completely
sk-ant-api03-abc123...

# Correct
ANTHROPIC_API_KEY=sk-ant-api03-abc123...
```
