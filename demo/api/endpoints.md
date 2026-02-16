# API Endpoints

Base URL: `https://api.acme.internal/v1`

## Services

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/services` | List all services |
| `POST` | `/services` | Register a new service |
| `GET` | `/services/:name` | Get service details |
| `PUT` | `/services/:name` | Update service config |
| `DELETE` | `/services/:name` | Deregister a service |

### List Services

```bash
curl https://api.acme.internal/v1/services \
  -H "Authorization: Bearer $TOKEN"
```

```json
{
  "services": [
    {
      "name": "payment-api",
      "version": "3.2.1",
      "status": "healthy",
      "replicas": 3,
      "endpoint": "https://payment-api.production.acme.internal"
    },
    {
      "name": "user-service",
      "version": "1.8.0",
      "status": "healthy",
      "replicas": 2,
      "endpoint": "https://user-service.production.acme.internal"
    }
  ],
  "total": 2
}
```

### Get Service Details

```bash
curl https://api.acme.internal/v1/services/payment-api \
  -H "Authorization: Bearer $TOKEN"
```

```json
{
  "name": "payment-api",
  "version": "3.2.1",
  "runtime": "rust",
  "status": "healthy",
  "replicas": { "ready": 3, "desired": 3 },
  "endpoint": "https://payment-api.production.acme.internal",
  "deploy": {
    "strategy": "canary",
    "last_deployed": "2026-02-10T14:32:00Z",
    "deployed_by": "jane.doe@acme.co"
  },
  "resources": {
    "cpu": "500m",
    "memory": "512Mi"
  }
}
```

## Deployments

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/services/:name/deploy` | Trigger a deployment |
| `GET` | `/services/:name/deploys` | List deploy history |
| `POST` | `/services/:name/rollback` | Rollback to previous version |

## Metrics

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/services/:name/metrics` | Current metrics snapshot |
| `GET` | `/services/:name/metrics/history` | Historical metrics |
