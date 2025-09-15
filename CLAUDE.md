# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Environment

This is a Rust web application using Nix flakes for development environment management.

### Getting Started

Use the Nix development shell:
```bash
nix develop
```

The shell hook will automatically run `cargo sqlx prepare` to prepare database queries.

### Building and Running

**Development:**
```bash
cargo run
```

**Production build with Nix:**
```bash
nix build
```

**Build docker image:**
```bash
nix build .#dockerImage
```

### Code Quality

**Format code:**
```bash
cargo fmt
```

**Lint code:**
```bash
cargo clippy
```

**Check with warnings as errors:**
```bash
RUSTFLAGS="-D warnings" cargo build
```

### Database

The application uses SQLite with migrations in the `migrations/` directory.

**Initial database setup:**
```bash
touch ./db/hitster.db
cargo sqlx migrate run
cargo sqlx prepare
```

**Development database URL is set in shell hook:** `sqlite://./db/hitster.db`

## Architecture

This is a web application for generating PDF cards from Spotify playlists, following Clean Architecture principles.

### Key Components

**Domain Layer (`src/domain/`):**
- Core business entities: `Playlist`, `Job`, `Pdf`, `SpotifyId`
- Value objects and domain logic
- No external dependencies

**Application Layer (`src/application/`):**
- `PlaylistService`: Main application service with trait interface
- `PdfGenerator`: Handles PDF generation
- `Worker`: Background job processing system
- Interfaces for repositories and external services

**Infrastructure Layer (`src/infrastructure/`):**
- `SpotifyClient`: Spotify API integration using rspotify
- `PlaylistRepository`: SQLite database operations
- `JobsRepository`: Background job persistence
- Database entities and SQLx queries

**Web Layer (`src/web/`):**
- Axum-based web server
- Askama templating for HTML view****s
- REST API controllers
- File serving for generated PDFs

### Background Jobs

The application uses a custom worker system for asynchronous operations:
- `GeneratePlaylistPdfsTask`: Creates front/back PDF cards for playlists
- `RefetchPlaylistTask`: Updates playlist data from Spotify API
- Jobs are persisted in SQLite and processed in background workers

### Database Schema

The application uses two main database entities:
- `playlists`: Stores playlist metadata and track information
- `jobs`: Tracks background job status and results

### Configuration

Required environment variables:
- `SPOTIFY_CLIENT_ID`: Spotify application client ID
- `SPOTIFY_CLIENT_SECRET`: Spotify application client secret

Optional variables:
- `DATABASE_URL`: SQLite database path (defaults to `sqlite://./db/hitster.db`)
- `HITSTER_HOST`: Server host (defaults to `127.0.0.1`)
- `HITSTER_PORT`: Server port (defaults to `3000`)
- `DATABASE_POOL_MAX_CONNECTIONS`: Connection pool size (defaults to `10`)
- `DATABASE_POOL_TIMEOUT_SECONDS`: Connection timeout (defaults to `30`)

### Dependencies

**Core:** Tokio for async runtime, Axum for web framework, SQLx for database
**Spotify:** rspotify for API integration
**PDF:** oxidize-pdf for PDF generation, qrcode for QR codes
**Templating:** askama for HTML templates
**Utilities:** serde, anyhow, tracing, uuid, chrono