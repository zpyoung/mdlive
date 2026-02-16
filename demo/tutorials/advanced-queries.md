# Advanced Queries

Use the Acme API's query parameters to filter, sort, and paginate results efficiently.

## Filtering

Most list endpoints accept filter parameters:

```bash
# services with status "healthy"
curl "https://api.acme.internal/v1/services?status=healthy"

# services running Python
curl "https://api.acme.internal/v1/services?runtime=python"

# combine filters
curl "https://api.acme.internal/v1/services?status=healthy&runtime=rust"
```

## Sorting

Use `sort` and `order` parameters:

```bash
# newest first
curl "https://api.acme.internal/v1/services?sort=created_at&order=desc"

# alphabetical by name
curl "https://api.acme.internal/v1/services?sort=name&order=asc"
```

## Pagination

The API uses cursor-based pagination for consistent results:

```bash
# first page (default 20 items)
curl "https://api.acme.internal/v1/services?limit=10"
```

Response includes pagination metadata:

```json
{
  "services": [...],
  "pagination": {
    "total": 47,
    "limit": 10,
    "next_cursor": "eyJpZCI6IjEyMyJ9",
    "has_more": true
  }
}
```

Fetch the next page:

```bash
curl "https://api.acme.internal/v1/services?limit=10&cursor=eyJpZCI6IjEyMyJ9"
```

## Full-Text Search

Search across service names and descriptions:

```bash
curl "https://api.acme.internal/v1/services?q=payment"
```

## Deployment History Queries

```bash
# last 5 deploys for a service
curl "https://api.acme.internal/v1/services/payment-api/deploys?limit=5"

# failed deploys in production
curl "https://api.acme.internal/v1/services/payment-api/deploys?environment=production&status=failed"

# deploys by a specific user
curl "https://api.acme.internal/v1/services/payment-api/deploys?deployed_by=jane.doe@acme.co"
```

## Field Selection

Reduce response size by selecting only the fields you need:

```bash
curl "https://api.acme.internal/v1/services?fields=name,version,status"
```

```json
{
  "services": [
    { "name": "payment-api", "version": "3.2.1", "status": "healthy" },
    { "name": "user-service", "version": "1.8.0", "status": "healthy" }
  ]
}
```
