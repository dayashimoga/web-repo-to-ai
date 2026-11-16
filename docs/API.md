# REST & Streaming API Specification (`docs/API.md`)

**Project:** `web-repo-to-ai`  
**Base Path:** `/api/v1`  
**Status:** Baseline Contract v1.0

---

## 1. Endpoints Overview

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/health` | Liveness probe (process up). |
| `GET` | `/api/v1/ready` | Readiness probe (PostgreSQL & storage reachable). |
| `POST` | `/api/v1/sources/detect` | Validate and classify one or more input URLs (webpage, public GitHub, private GitHub, batch). |
| `POST` | `/api/v1/jobs` | Create a new conversion job with source URL(s), target ref/filters, optimization profile, token budget, and export formats. |
| `GET` | `/api/v1/jobs` | List conversion jobs for the current session/user with pagination. |
| `GET` | `/api/v1/jobs/{job_id}` | Retrieve job status, completeness summary, token statistics, warnings, and artifact metadata. |
| `GET` | `/api/v1/jobs/{job_id}/events` | Server-Sent Events (SSE) stream emitting real-time job phase, file accounting, and completion events. |
| `POST` | `/api/v1/jobs/{job_id}/cancel` | Cancel an active or queued conversion job. |
| `GET` | `/api/v1/jobs/{job_id}/preview` | Retrieve paginated/virtualized preview of the converted output and repository tree. |
| `GET` | `/api/v1/jobs/{job_id}/artifacts/{format}` | Stream download of a generated artifact (`markdown`, `txt`, `json`, `html`, `jsonl`, `zip`, `manifest`). |
| `GET` | `/api/v1/github/auth/start` | Initiate GitHub App OAuth / installation flow with signed CSRF state. |
| `GET` | `/api/v1/github/auth/callback` | Handle GitHub App OAuth / installation callback. |
| `GET` | `/api/v1/github/repositories` | List authorized repositories for the user's active GitHub App installations. |
| `POST` | `/api/v1/github/disconnect` | Revoke and purge stored GitHub installation credentials for the current user. |

---

## 2. Standard Error Contract

All API error responses use a consistent, machine-readable JSON envelope (RFC 7807 style):

```json
{
  "error": {
    "code": "SSRF_ADDRESS_BLOCKED",
    "message": "The requested URL resolved to a restricted private or loopback network address.",
    "details": {
      "url": "http://127.0.0.1:8080/admin"
    },
    "request_id": "019335c4-7c20-7000-8000-000000000001"
  }
}
```

### Standard Error Codes
- `INVALID_REQUEST_SCHEMA` (`400`)
- `UNSUPPORTED_URL_SCHEME` (`400`)
- `SSRF_ADDRESS_BLOCKED` (`403`)
- `UNAUTHORIZED_GITHUB_REPO` (`401` / `403`)
- `RESOURCE_NOT_FOUND` (`404`)
- `RATE_LIMIT_EXCEEDED` (`429`)
- `PAYLOAD_TOO_LARGE` (`413`)
- `EXTRACTION_TIMEOUT` (`504`)
- `INTERNAL_SERVER_ERROR` (`500`)
