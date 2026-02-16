# Authentication

All API requests require a bearer token obtained via OAuth 2.0.

## Obtaining a Token

```bash
curl -X POST https://auth.acme.internal/oauth/token \
  -H "Content-Type: application/json" \
  -d '{"grant_type": "client_credentials", "client_id": "YOUR_ID", "client_secret": "YOUR_SECRET"}'
```

Response:

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type": "bearer",
  "expires_in": 3600
}
```

## Using the Token

Include the token in the `Authorization` header:

```bash
curl https://api.acme.internal/v1/services \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIs..."
```

## Token Refresh

Tokens expire after one hour. Request a new token before expiration. The CLI handles this automatically.

## Scopes

| Scope | Access |
|-------|--------|
| `read:services` | List and describe services |
| `write:services` | Create, update, delete services |
| `deploy` | Trigger deployments |
| `admin` | Manage users and roles |

Request scopes during token creation:

```bash
curl -X POST https://auth.acme.internal/oauth/token \
  -d '{"grant_type": "client_credentials", "client_id": "YOUR_ID", "client_secret": "YOUR_SECRET", "scope": "read:services deploy"}'
```

## Service Accounts

For CI/CD pipelines, create a service account:

```bash
acme iam create-service-account --name ci-deployer --scopes deploy,read:services
```

This returns a client ID and secret. Store them in your CI environment variables.
