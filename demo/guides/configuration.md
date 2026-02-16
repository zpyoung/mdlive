# Configuration

Every Acme service is configured through `acme.yaml` at the project root.

## Minimal Config

```yaml
name: my-api
version: "1.0.0"
runtime: python
port: 8000
```

## Full Reference

```yaml
name: my-api
version: "1.0.0"
runtime: python           # python | rust | node | go
port: 8000

build:
  dockerfile: Dockerfile
  context: .
  args:
    PYTHON_VERSION: "3.12"

deploy:
  replicas: 2
  strategy: rolling        # rolling | canary | blue-green | recreate
  health_check:
    path: /health
    interval: 10s
    timeout: 5s
    threshold: 3

resources:
  cpu: 500m
  memory: 512Mi
  limits:
    cpu: "1"
    memory: 1Gi

scaling:
  enabled: false
  min_replicas: 2
  max_replicas: 10
  target_cpu: 70

env:
  DATABASE_URL: ${vault:db/my-api/url}
  REDIS_URL: redis://cache.internal:6379/0
  LOG_LEVEL: info

dependencies:
  - name: postgresql
    version: "15"
  - name: redis
    version: "7"
```

## Environment Variables

Variables can reference Vault secrets using the `${vault:path}` syntax. The deploy engine resolves these at deploy time.

```yaml
env:
  SECRET_KEY: ${vault:app/my-api/secret_key}
  DB_PASSWORD: ${vault:db/my-api/password}
```

## Per-Environment Overrides

Create `acme.staging.yaml` or `acme.production.yaml` to override values per environment. Only the fields you specify are merged:

```yaml
# acme.staging.yaml
deploy:
  replicas: 1

resources:
  cpu: 250m
  memory: 256Mi
```

## Validation

```bash
$ acme validate
acme.yaml ............ valid
acme.staging.yaml .... valid
Dockerfile ........... valid
```
