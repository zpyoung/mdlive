# Rate Limiting

The API enforces rate limits to protect shared infrastructure.

Good times
Great times

## Limits

| Tier | Requests/min | Burst |
|------|-------------|-------|
| Standard | 60 | 10 |
| Deployer | 120 | 20 |
| Admin | 300 | 50 |

Your tier is determined by the scopes on your token.

## Headers

Every response includes rate limit headers:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 42
X-RateLimit-Reset: 1708100400
```

## Handling 429 Responses

When you exceed the limit, the API returns `429 Too Many Requests` with a `Retry-After` header:

```
HTTP/1.1 429 Too Many Requests
Retry-After: 30
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1708100400
```

Implement exponential backoff in your client:

```python
import time
import requests

def api_call_with_retry(url, headers, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url, headers=headers)
        if response.status_code != 429:
            return response

        retry_after = int(response.headers.get("Retry-After", 2 ** attempt))
        time.sleep(retry_after)

    raise Exception("Rate limit exceeded after retries")
```

## Best Practices

1. Cache responses where possible -- service details rarely change
2. Use webhooks instead of polling for deploy status
3. Batch operations where the API supports it
4. Spread requests evenly rather than bursting at interval boundaries
