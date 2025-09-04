# Hitster Web API Documentation

## Overview

The Hitster web API provides endpoints for managing Spotify playlists and generating PDF cards from them. The API follows RESTful conventions and returns JSON responses.

## Base URL

```
http://localhost:8080
```

## Authentication

Currently, no authentication is required for API endpoints.

## Response Format

All API responses are in JSON format. Success responses include a `status` field, while errors return appropriate HTTP status codes and error messages.

## Error Handling

The API returns standard HTTP status codes:

- `200 OK` - Successful request
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

Error responses include:
```json
{
  "error": "Error message describing what went wrong",
  "details": "Additional error details if available"
}
```

## Endpoints

### View Endpoints

#### GET /view/
Serve the main index page with playlist URL input form.

**Response:** HTML content

#### GET /view/playlist/:playlist_id
Serve the cards page for a specific playlist, showing track information and PDF generation status.

**Parameters:**
- `playlist_id` (path, required): The playlist identifier

**Response:** HTML content

### Playlist Management

#### POST /api/playlist
Create a new playlist from a Spotify URL.

**Request Body:**
```json
{
  "url": "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"
}
```

**Response (201 Created):**
```json
{
  "playlist_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Playlist created successfully",
  "status": "success"
}
```

**Errors:**
- `400 Bad Request` - Invalid or empty URL
- `500 Internal Server Error` - Failed to process playlist

#### GET /api/playlist/:id
Get detailed information about a playlist.

**Parameters:**
- `id` (path, required): The playlist identifier

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "My Awesome Playlist",
  "track_count": 25,
  "tracks": [
    {
      "title": "Song Title",
      "artist": "Artist Name",
      "year": "2023",
      "spotify_url": "https://open.spotify.com/track/...",
      "duration_ms": 180000,
      "album": "Album Name",
      "popularity": 85
    }
  ],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

**Errors:**
- `404 Not Found` - Playlist not found
- `400 Bad Request` - Invalid playlist ID format

#### POST /api/playlist/:id/refetch-playlist
Refetch playlist information from Spotify.

**Parameters:**
- `id` (path, required): The playlist identifier

**Response (200 OK):**
```json
{
  "message": "Playlist refetched successfully",
  "status": "success"
}
```

**Errors:**
- `404 Not Found` - Playlist not found
- `400 Bad Request` - Invalid playlist ID format

### PDF Generation

#### POST /api/playlist/:id/pdfs
Generate PDF cards for a playlist.

**Parameters:**
- `id` (path, required): The playlist identifier

**Response (200 OK):**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "PDF generation job created",
  "status": "success",
  "estimated_completion_ms": 30000
}
```

**Errors:**
- `404 Not Found` - Playlist not found
- `400 Bad Request` - Invalid playlist ID format

#### GET /api/playlist/:id/pdfs/:side
Download generated PDF for a playlist.

**Parameters:**
- `id` (path, required): The playlist identifier
- `side` (path, required): PDF side ("front" or "back")

**Response:** PDF file download with appropriate headers

**Errors:**
- `404 Not Found` - PDF not available
- `400 Bad Request` - Invalid parameters

### Job Management

#### GET /api/jobs/:job_id
Get detailed information about a job.

**Parameters:**
- `job_id` (path, required): The job identifier

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "playlist_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Completed",
  "status_description": "Processing completed successfully",
  "created_at": "2023-01-01T00:00:00Z",
  "completed_at": "2023-01-01T00:01:00Z",
  "front_pdf_path": "/path/to/front.pdf",
  "back_pdf_path": "/path/to/back.pdf",
  "job_type": "GeneratePlaylistPdf",
  "progress_percent": 100,
  "error_message": null
}
```

**Errors:**
- `404 Not Found` - Job not found
- `400 Bad Request` - Invalid job ID format

#### GET /api/jobs/:job_id/status
Get the current status of a job.

**Parameters:**
- `job_id` (path, required): The job identifier

**Response (200 OK):**
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "Processing",
  "status_description": "Currently processing",
  "is_completed": false,
  "progress_percent": 50,
  "estimated_time_remaining_seconds": 30,
  "can_cancel": true
}
```

**Errors:**
- `404 Not Found` - Job not found
- `400 Bad Request` - Invalid job ID format

#### GET /api/jobs/:job_id/pdf/:side
Download generated PDF for a job.

**Parameters:**
- `job_id` (path, required): The job identifier
- `side` (path, required): PDF side ("front" or "back")

**Response:** PDF file download with appropriate headers

**Errors:**
- `404 Not Found` - PDF not available
- `400 Bad Request` - Invalid parameters

## Data Models

### Job Status Values

Jobs can have the following statuses:
- `Pending` - Waiting to start processing
- `Processing` - Currently processing
- `Completed` - Processing completed successfully
- `Failed` - Processing failed

### PDF Side Values

PDFs can be generated for two sides:
- `front` - Front side of the cards
- `back` - Back side of the cards

## Rate Limiting

Currently, no rate limiting is implemented.

## Examples

### Creating a Playlist

```bash
curl -X POST http://localhost:8080/api/playlist \
  -H "Content-Type: application/json" \
  -d '{"url": "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"}'
```

### Generating PDFs

```bash
curl -X POST http://localhost:8080/api/playlist/550e8400-e29b-41d4-a716-446655440000/pdfs
```

### Checking Job Status

```bash
curl http://localhost:8080/api/jobs/550e8400-e29b-41d4-a716-446655440000/status
```

## Legacy Endpoints

The following legacy endpoints are still supported for backward compatibility:

- `GET /` - Index page
- `POST /submit-playlist` - Submit playlist form
- `GET /cards/:playlist_id` - Cards page
- `POST /api/generate/:job_id` - Start PDF generation
- `GET /api/jobs/:job_id` - Get job status
- `GET /api/download/:playlist_id/:type` - Download PDF

## Version History

- **v1.0.0** - Initial API release with clean architecture
- **v0.1.0** - Legacy implementation

## Support

For issues and feature requests, please refer to the project repository.