# Architecture Overview

Acme Platform is composed of five core services behind a single API gateway.

```mermaid
graph TB
    subgraph External
        CLI[acme CLI]
        Dashboard[Web Dashboard]
        CI[CI/CD Pipelines]
    end

    subgraph Gateway Layer
        GW[API Gateway<br/>nginx + lua]
    end

    subgraph Core Services
        Auth[Auth Service<br/>Rust / Axum]
        Catalog[Service Catalog<br/>Rust / Axum]
        Deploy[Deploy Engine<br/>Go]
        Monitor[Monitor<br/>Python / FastAPI]
        Webhook[Webhook Dispatcher<br/>Rust]
    end

    subgraph Data
        PG[(PostgreSQL 15)]
        Redis[(Redis 7)]
        S3[S3 - Artifacts]
        Prom[Prometheus]
    end

    subgraph Infrastructure
        K8s[Kubernetes 1.29]
        Reg[Container Registry]
    end

    CLI --> GW
    Dashboard --> GW
    CI --> GW

    GW --> Auth
    GW --> Catalog
    GW --> Deploy
    GW --> Monitor
    GW --> Webhook

    Auth --> Redis
    Catalog --> PG
    Deploy --> K8s
    Deploy --> Reg
    Deploy --> S3
    Monitor --> Prom
    Webhook --> Redis
```

## Service Responsibilities

**Auth Service** handles OAuth 2.0 token issuance, validation, scope enforcement, and service account management. Tokens are JWTs signed with RS256, validated at the gateway layer.

**Service Catalog** is the source of truth for registered services, their configurations, and runtime metadata. Backed by PostgreSQL with a simple versioned schema.

**Deploy Engine** orchestrates deployments across Kubernetes clusters. It supports rolling, blue-green, canary, and recreate strategies. Each deploy creates an immutable snapshot for rollback.

**Monitor** aggregates metrics from Prometheus and exposes them through the API and dashboard. It also evaluates alert rules and fires webhook events.

**Webhook Dispatcher** delivers event notifications to registered endpoints with retry logic and signature verification.

## Communication Patterns

All inter-service communication is synchronous HTTP/gRPC through the service mesh. The only async path is the webhook dispatcher, which uses a Redis-backed queue for reliable delivery.
