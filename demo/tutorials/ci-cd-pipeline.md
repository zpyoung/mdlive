# CI/CD Pipeline

Set up automated testing and deployment with GitLab CI.

## Pipeline Overview

```mermaid
graph LR
    Commit[Push to<br/>GitLab] --> Lint[Lint &<br/>Format]
    Lint --> Test[Unit &<br/>Integration Tests]
    Test --> Build[Build<br/>Container]
    Build --> DeployStaging[Deploy to<br/>Staging]
    DeployStaging --> E2E[E2E<br/>Tests]
    E2E --> Approval{Manual<br/>Approval}
    Approval -->|approved| DeployProd[Deploy to<br/>Production]
    Approval -->|rejected| Stop[Pipeline<br/>Stops]
```

## GitLab CI Configuration

```yaml
# .gitlab-ci.yml
stages:
  - check
  - test
  - build
  - deploy-staging
  - e2e
  - deploy-production

variables:
  ACME_SERVICE: my-api

lint:
  stage: check
  script:
    - ruff check src/
    - ruff format --check src/

unit-tests:
  stage: test
  script:
    - pytest tests/unit/ -v --junitxml=report.xml
  artifacts:
    reports:
      junit: report.xml

integration-tests:
  stage: test
  services:
    - postgres:15
    - redis:7
  variables:
    DATABASE_URL: postgresql://test:test@postgres:5432/test
    REDIS_URL: redis://redis:6379/0
  script:
    - pytest tests/integration/ -v

build:
  stage: build
  script:
    - acme build
    - acme push

deploy-staging:
  stage: deploy-staging
  script:
    - acme deploy --env staging --strategy rolling
  environment:
    name: staging
    url: https://$ACME_SERVICE.staging.acme.internal

e2e-tests:
  stage: e2e
  script:
    - pytest tests/e2e/ -v --base-url https://$ACME_SERVICE.staging.acme.internal

deploy-production:
  stage: deploy-production
  script:
    - acme deploy --env production --strategy canary --weight 10
  environment:
    name: production
    url: https://$ACME_SERVICE.production.acme.internal
  when: manual
  only:
    - main
```

## Service Account Setup

Create a service account for CI:

```bash
acme iam create-service-account \
  --name ci-$ACME_SERVICE \
  --scopes deploy,read:services
```

Add the credentials as CI/CD variables in GitLab:

- `ACME_CLIENT_ID` - the service account client ID
- `ACME_CLIENT_SECRET` - the service account secret (masked)

## Branch Strategies

| Branch | Triggers | Deploys To |
|--------|----------|------------|
| Feature branches | lint, test | nowhere |
| `main` | lint, test, build | staging (auto), production (manual) |
| Tags (`v*`) | lint, test, build | staging, production (auto) |

## Notifications

Configure deploy notifications in `acme.yaml`:

```yaml
notifications:
  slack:
    channel: "#deploys"
    events: [deploy.completed, deploy.failed]
  email:
    recipients: ["team@acme.co"]
    events: [deploy.failed]
```
