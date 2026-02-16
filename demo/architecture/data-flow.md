# Data Flow

## Deployment Pipeline

From `acme deploy` to running pods:

```mermaid
sequenceDiagram
    actor Dev as Developer
    participant CLI as acme CLI
    participant GW as API Gateway
    participant Auth as Auth Service
    participant Deploy as Deploy Engine
    participant Reg as Container Registry
    participant K8s as Kubernetes
    participant WH as Webhook Dispatcher

    Dev->>CLI: acme deploy --env production
    CLI->>CLI: Build container image
    CLI->>Reg: Push image
    CLI->>GW: POST /v1/services/my-api/deploy
    GW->>Auth: Validate token
    Auth-->>GW: OK (scopes: deploy)
    GW->>Deploy: Forward request
    Deploy->>Deploy: Create deploy snapshot
    Deploy->>K8s: Apply manifests
    K8s-->>Deploy: Rollout status
    Deploy->>WH: Emit deploy.completed
    WH-->>Dev: Webhook notification
    Deploy-->>GW: 200 OK
    GW-->>CLI: Deploy successful
    CLI-->>Dev: Done
```

## Service Discovery Flow

When a service starts, it registers itself in the catalog:

1. Pod starts and passes health checks
2. Init container calls `POST /v1/services` with metadata from `acme.yaml`
3. Catalog stores entry in PostgreSQL
4. Other services query the catalog for endpoint resolution

```mermaid
graph LR
    Pod[New Pod] -->|health check passes| Init[Init Container]
    Init -->|POST /services| Catalog[Service Catalog]
    Catalog -->|store| PG[(PostgreSQL)]
    Client[Other Services] -->|GET /services/name| Catalog
    Catalog -->|return endpoint| Client
```

## Metrics Pipeline

```mermaid
graph LR
    App[Application] -->|expose /metrics| Prom[Prometheus]
    Prom -->|scrape every 15s| Prom
    Monitor[Monitor Service] -->|PromQL queries| Prom
    Monitor -->|evaluate rules| Monitor
    Monitor -->|alert.fired| WH[Webhook Dispatcher]
    Dashboard[Web Dashboard] -->|GET /metrics| Monitor
```

Prometheus scrapes application metrics every 15 seconds. The Monitor service runs PromQL queries against Prometheus to power the dashboard and evaluate alert rules. When a rule fires, it emits an event through the webhook dispatcher.
