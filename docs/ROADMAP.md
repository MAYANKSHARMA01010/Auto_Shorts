# AutoShorts Roadmap

This roadmap outlines potential future improvements and features for the AutoShorts project. Note that this project is currently considered feature-complete and highly stable. These are speculative optimizations.

## 1. Performance & Architecture Optimizations
- **WebGPU Video Rendering:** Offload FFmpeg processing natively to the GPU using WebCodecs or a Rust `wgpu` pipeline.
- **SQLite WAL Mode:** Enable Write-Ahead Logging to permit highly concurrent database access without requiring aggressive mutex locking.

## 2. LLM Capabilities
- **Multi-modal Review:** Allow the LLM to process frames of the video (via vision models) to find visually engaging segments, rather than relying solely on the textual transcript.
- **Auto-B-Roll:** Use the transcript to automatically fetch and overlay relevant stock footage on dull segments.

## 3. UI/UX Enhancements
- **Timeline Editor:** Build a rich, draggable timeline in Next.js to allow users to manually trim and adjust the boundaries of the AI-selected segments.
- **Custom Fonts:** Provide a UI to upload and select custom `.ttf`/`.otf` fonts for the generated VTT captions.
