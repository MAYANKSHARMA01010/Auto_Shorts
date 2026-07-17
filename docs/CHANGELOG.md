# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased] - Final Polish Mode
### Added
- Comprehensive architecture and API documentation (`docs/ARCHITECTURE.md`, `docs/API.md`).
- Centralized troubleshooting guide.
- New scripts directory for legacy workflow python patches.

### Changed
- Standardized package management around `pnpm` workspace.
- Refactored `package.json` configurations to proxy correctly into the frontend.
- Optimized and stabilized the FFmpeg rendering pipeline to completely eliminate deadlocks.
- Hardened SQLite database operations with thread-safe `Mutex` wrappers.

### Removed
- Dead code in `src-tauri` (unused dependencies like `thiserror`).
- Duplicate `icon.png` and obsolete `package-lock.json` lockfiles.
- Unnecessary frontend build caches (`out/`, `.next/`, `dist/`).

## [0.1.3] - Production Stabilization
### Added
- Viral score prediction logic.
- Full Next.js static HTML export support into Tauri.
- Support for local Ollama instances.

### Fixed
- Fixed a concurrency panic occurring during simultaneous database writes.
- Fixed a path resolution bug where FFmpeg could not read files containing spaces or special characters on macOS.
