# Webhooks

Receive real-time notifications for platform events instead of polling.

## Registering a Webhook

```bash
curl -X POST https://api.acme.internal/v1/webhooks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://my-service.internal/hooks/acme",
    "events": ["deploy.started", "deploy.completed", "deploy.failed"],
    "secret": "whsec_abc123"
  }'
```

## Events

| Event | Trigger |
|-------|---------|
| `deploy.started` | A deployment begins |
| `deploy.completed` | A deployment finishes successfully |
| `deploy.failed` | A deployment fails or is rolled back |
| `service.registered` | A new service appears in the catalog |
| `service.deregistered` | A service is removed from the catalog |
| `scaling.triggered` | Auto-scaling changes replica count |
| `alert.fired` | A monitoring alert fires |

## Payload Format

```json
{
  "id": "evt_9a8b7c6d",
  "event": "deploy.completed",
  "timestamp": "2026-02-10T14:35:00Z",
  "data": {
    "service": "payment-api",
    "version": "3.2.1",
    "environment": "production",
    "strategy": "canary",
    "duration_seconds": 180,
    "deployed_by": "jane.doe@acme.co"
  }
}
```

## Verifying Signatures

Every webhook request includes an `X-Acme-Signature` header. Verify it using HMAC-SHA256:

```python
import hmac
import hashlib

def verify_signature(payload: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(
        secret.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

## Retry Policy

Failed deliveries (non-2xx response) are retried with exponential backoff:

- Attempt 1: immediate
- Attempt 2: after 1 minute
- Attempt 3: after 5 minutes
- Attempt 4: after 30 minutes
- Attempt 5: after 2 hours

After 5 failed attempts, the webhook is marked as disabled. Re-enable it via the API or dashboard.

## Managing Webhooks

```bash
# list webhooks
curl https://api.acme.internal/v1/webhooks -H "Authorization: Bearer $TOKEN"

# delete a webhook
curl -X DELETE https://api.acme.internal/v1/webhooks/wh_abc123 -H "Authorization: Bearer $TOKEN"
```
