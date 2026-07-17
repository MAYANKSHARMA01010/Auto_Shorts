# Troubleshooting Guide

This document lists common issues you might encounter while running or developing AutoShorts, and how to resolve them.

## 1. FFmpeg or FFprobe Not Found
**Error:** `ffmpeg is not installed or not available on PATH`
**Solution:** AutoShorts relies heavily on `ffmpeg` and `ffprobe` for media processing.
- **Mac:** `brew install ffmpeg`
- **Windows:** Download from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) or use `winget install ffmpeg`. Ensure the `bin/` directory is added to your System PATH variables.
- **Linux:** `sudo apt install ffmpeg`

## 2. API Rate Limits & Timeouts
**Error:** `DeepSeek request failed (429)` or Network Timeout.
**Solution:**
- The application processes very large blocks of text during the `generate_clips` phase. Ensure your API provider account has sufficient credits and isn't hitting strict requests-per-minute (RPM) limits.
- If timeouts persist, switch to a faster or locally hosted model like `Ollama`.

## 3. Empty Transcript Generated
**Error:** The transcript returns with 0 words.
**Solution:**
- Check if the uploaded video file contains a valid audio track.
- If using Deepgram, verify your `DEEPGRAM_API_KEY` is correct in `.env`.
- Some exceptionally quiet or corrupted audio files might fail to parse. You can verify this by playing the file locally.

## 4. UI Doesn't Connect to Backend (Fetch Failed)
**Error:** The frontend throws a fetch error or fails to load projects.
**Solution:**
- AutoShorts uses Tauri commands instead of `fetch`. Ensure you are running the app through the Tauri runtime (`pnpm run tauri:dev`), **not** just `next dev`. Next.js standalone cannot access the Tauri bridge.

## 5. Corrupted `.vtt` Export
**Error:** The burned captions contain strange symbols or the render fails.
**Solution:**
- Ensure your source video doesn't contain incompatible codec metadata. AutoShorts re-encodes audio to AAC and video to H.264 automatically, but heavily corrupted files might fail the subtitle filter (`subtitles=...`).
