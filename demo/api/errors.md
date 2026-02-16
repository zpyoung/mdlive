# Error Handling

The API returns standard HTTP status codes with a consistent error body.

## Error Format

```json
{
  "error": {
    "code": "service_not_found",
    "message": "No service named 'foo-api' found in the catalog.",
    "request_id": "req_7f3a2b1c"
  }
}
```

## Status Codes

| Code | Meaning |
|------|---------|
| `400` | Bad request -- malformed JSON or missing required fields |
| `401` | Unauthorized -- missing or expired token |
| `403` | Forbidden -- token lacks required scope |
| `404` | Not found -- resource doesn't exist |
| `409` | Conflict -- resource already exists or deploy in progress |
| `422` | Unprocessable -- validation failed (details in `message`) |
| `429` | Rate limited -- slow down, see [rate limiting](rate-limiting.md) |
| `500` | Internal error -- file a bug with the `request_id` |

## Common Errors

### Invalid Manifest

```json
{
  "error": {
    "code": "validation_failed",
    "message": "deploy.strategy must be one of: rolling, canary, blue-green, recreate",
    "request_id": "req_9e4d3c2a"
  }
}
```

### Deploy Conflict

Returned when a deploy is already in progress for the service:

```json
{
  "error": {
    "code": "deploy_in_progress",
    "message": "A deployment for 'payment-api' is already running. Wait for it to complete or cancel it.",
    "request_id": "req_1a2b3c4d"
  }
}
```

### Rate Limited

```json
{
  "error": {
    "code": "rate_limited",
    "message": "Rate limit exceeded. Retry after 30 seconds.",
    "retry_after": 30,
    "request_id": "req_5e6f7a8b"
  }
}
```

## Request IDs

Every response includes an `X-Request-Id` header. Include this when reporting issues to the platform team.
