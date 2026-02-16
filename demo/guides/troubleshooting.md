# Troubleshooting

Common issues and how to fix them.

## Deploy Fails with "Health Check Timeout"

The deploy engine waits for the health check endpoint to return 200. If it doesn't within the threshold, the deploy is rolled back.

**Check the logs:**

```bash
acme logs my-api --env staging --tail 100
```

**Common causes:**

1. The application crashes on startup -- look for stack traces in logs
2. The health check path is wrong in `acme.yaml`
3. The port in `acme.yaml` doesn't match what the app actually listens on
4. A dependency (database, Redis) is unreachable from the cluster

## "Permission Denied" on Deploy

Your token may have expired or lack the required role.

```bash
$ acme whoami
user:    jane.doe@acme.co
roles:   developer
expires: 2026-01-01 (EXPIRED)

$ acme login   # re-authenticate
```

If your roles don't include `deployer`, request access in the #acme-platform channel.

## Container Build Fails

```bash
# build locally to see full output
acme build --verbose

# common fix: update base image
docker pull python:3.12-slim
```

## WebSocket Disconnects in Dashboard

The dashboard uses WebSocket connections for live updates. If you see frequent disconnects:

- Check if a corporate proxy is terminating long-lived connections
- Try switching to the polling transport: append `?transport=polling` to the dashboard URL
- Verify the service mesh sidecar isn't timing out idle connections (default: 60s)

## "Service Not Found" in Catalog

After deploying, the service takes up to 30 seconds to register in the catalog. If it persists:

```bash
acme catalog refresh
acme catalog list | grep my-api
```

If the service still doesn't appear, check that `name` in `acme.yaml` matches exactly.

## Getting More Help

```bash
acme doctor          # run diagnostics
acme support-bundle  # generate a debug archive to share with the platform team
```
