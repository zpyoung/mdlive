# Quickstart

Get a service running on the Acme Platform in under five minutes.

## Prerequisites

- macOS or Linux
- Docker 24+
- Access to the internal network (VPN if remote)

## Install the CLI

```bash
curl -sSL https://acme.internal/install | sh
```

Verify the installation:

```bash
$ acme version
acme-cli 2.1.0 (darwin/arm64)
```

## Authenticate

```bash
acme login
```

This opens a browser window for SSO. After authentication, your token is stored in `~/.acme/credentials`.

## Create a Service

```bash
acme init my-api --template=python-fastapi
cd my-api
```

The generated project includes:

```
my-api/
  src/
    main.py
    routes/
      health.py
  tests/
    test_health.py
  Dockerfile
  acme.yaml
  pyproject.toml
```

## Deploy to Staging

```bash
acme deploy --env staging
```

The CLI builds the container image, pushes it to the registry, and creates a Kubernetes deployment. Watch the rollout:

```bash
$ acme status my-api --env staging
Service:     my-api
Environment: staging
Replicas:    2/2 ready
Endpoint:    https://my-api.staging.acme.internal
Health:      passing
```

## Next Steps

- Read the [deployment guide](guides/deployment.md) for canary and blue-green strategies
- Set up [monitoring](tutorials/monitoring.md) for your service
- Browse the [API reference](api/endpoints.md) for the platform API

> **Tip:** Run `acme --help` for a full list of commands, or `acme <command> --help` for details on any subcommand.
