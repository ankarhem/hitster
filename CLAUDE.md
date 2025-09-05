# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hitster is a Rust web application that generates printable cards from Spotify playlists. The application follows a clean architecture pattern with distinct layers for domain logic, application services, infrastructure, and web presentation.

## Architecture

The codebase follows a hexagonal architecture with clear separation of concerns:

- **Domain Layer** (`src/domain/`): Core business entities and value objects
  - `job.rs`: Job processing entities
  - `pdf.rs`: PDF generation domain models
  - `playlist.rs`: Playlist and track entities

- **Application Layer** (`src/application/`): Use cases and business logic
  - `job_service.rs`: Job orchestration
  - `playlist_service.rs`: Playlist processing and PDF generation
  - `interfaces/`: Abstract interfaces for infrastructure

- **Infrastructure Layer** (`src/infrastructure/`): External integrations
  - `spotify/`: Spotify API client implementation
  - `jobs/`: Job persistence (currently todo)
  - `playlist/`: Playlist data persistence

- **Web Layer** (`src/web/`): HTTP server and presentation
  - `controllers/`: Request handlers
  - `templates/`: Askama templates (the corresponding html templates are in the `templates/` directory in root)
  - `server.rs`: Axum web server setup

## Development Environment

The project uses Nix for development environment management. Enter the development shell with:

```bash
nix develop
```

Or use standard Rust toolchain:

```bash
cargo build
cargo run
```

## Configuration

The application supports multiple configuration sources in priority order:

1. Environment variables (with `HITSTER_` prefix)
2. `.env` file
3. `config.toml` file

Required configuration:
- `SPOTIFY_CLIENT_ID`: Spotify API client ID
- `SPOTIFY_CLIENT_SECRET`: Spotify API client secret

## Common Commands

### Building and Running
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run the web server
cargo run

# Run with hot reloading
cargo watch -x run
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Check compilation (fast)
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Fix clippy issues
cargo clippy --fix
```

### Database Operations
```bash
# The database migrations are run authomatically on startup, but it is possible to run them manually too if necessary.
sqlx migrate run

# Add new migration
sqlx migrate add migration_name

# Reset database
rm db/hitster.db
sqlx migrate run
```

## Key Components

### Spotify Integration
- Uses `rspotify` crate for Spotify API integration
- Handles OAuth2 authentication
- Fetches playlist and track metadata

### PDF Generation
- Generates front and back card PDFs with QR codes
- Uses QR codes linking directly to Spotify track**s
- Templates in `templates/` directory for HTML rendering

### Jobs (todo)
- Asynchronous job processing for PDF generation
- Saved to database (implementation pending)
- Running as a background task (implementation pending)

### Web Server
- Axum-based web server
- Serves HTML templates for playlist management
- RESTful API for job and playlist operations

## Database Schema

Uses SQLite with migrations in `migrations/`. The schema supports:
- Playlists with Spotify metadata
- Jobs for tracking PDF generation
- PDF file storage references

## Current Status

The application is under active development with some incomplete implementations. Several infrastructure components have placeholder implementations that log warnings about unused variables.