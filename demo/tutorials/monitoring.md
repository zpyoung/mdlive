# Monitoring

Set up observability for your Acme Platform services.

## Metrics

Every service gets basic metrics for free via the service mesh sidecar:

- Request rate, latency (p50/p95/p99), and error rate
- CPU and memory usage
- Active connections

### Custom Metrics

Expose application-specific metrics on `/metrics` in Prometheus format:

```python
from prometheus_client import Counter, Histogram, generate_latest
from fastapi import FastAPI, Response

app = FastAPI()

REQUEST_COUNT = Counter(
    "bookmarks_requests_total",
    "Total bookmark requests",
    ["method", "endpoint", "status"]
)

REQUEST_LATENCY = Histogram(
    "bookmarks_request_duration_seconds",
    "Request latency in seconds",
    ["endpoint"]
)

@app.get("/metrics")
def metrics():
    return Response(
        content=generate_latest(),
        media_type="text/plain"
    )
```

## Dashboards

The Acme dashboard automatically generates a service overview. Access it at:

```
https://dashboard.acme.internal/services/my-api
```

### Key Panels

| Panel | Shows | Alert Threshold |
|-------|-------|----------------|
| Request Rate | req/s over time | n/a |
| Error Rate | 5xx / total | > 1% for 5 min |
| P95 Latency | 95th percentile response time | > 500ms for 5 min |
| CPU Usage | cores used vs limit | > 80% for 10 min |
| Memory Usage | RSS vs limit | > 85% for 10 min |
| Pod Restarts | restart count over 24h | > 3 in 1 hour |

## Alerts

Define alert rules in `acme.yaml`:

```yaml
alerts:
  - name: high_error_rate
    expr: "rate(http_requests_total{status=~'5..'}[5m]) / rate(http_requests_total[5m]) > 0.01"
    for: 5m
    severity: critical
    notify:
      - slack:#oncall
      - pagerduty

  - name: high_latency
    expr: "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5"
    for: 5m
    severity: warning
    notify:
      - slack:#my-api-alerts

  - name: pod_restart_loop
    expr: "increase(kube_pod_container_status_restarts_total[1h]) > 3"
    for: 0m
    severity: critical
    notify:
      - slack:#oncall
      - pagerduty
```

## Distributed Tracing

Traces are collected automatically via the service mesh. Add custom spans for application-level visibility:

```python
from opentelemetry import trace

tracer = trace.get_tracer(__name__)

async def process_bookmark(data):
    with tracer.start_as_current_span("process_bookmark") as span:
        span.set_attribute("bookmark.url", data.url)
        span.set_attribute("bookmark.tags_count", len(data.tags))

        with tracer.start_as_current_span("validate"):
            validate(data)

        with tracer.start_as_current_span("persist"):
            await save_to_db(data)
```

View traces in the dashboard or query via CLI:

```bash
acme trace --service my-api --last 15m --min-duration 100ms
```

## Runbook Template

For each alert, maintain a runbook entry:

```markdown
### high_error_rate

**Severity:** Critical
**Impact:** Users may see 500 errors

**Triage steps:**
1. Check recent deploys: `acme deploys my-api --last 5`
2. Review logs: `acme logs my-api --tail 200 --level error`
3. Check dependency health: `acme status --deps my-api`
4. If caused by a deploy, rollback: `acme rollback my-api`
```
