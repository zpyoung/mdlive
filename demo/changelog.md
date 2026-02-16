# Changelog

All notable changes to Acme Platform.

## v2.1.0 - 2026-02-10

### Added

- Auto-scaling policies with custom metric targets
- `acme scale` command for manual scaling overrides
- Dashboard widget for real-time pod count

### Changed

- Deploy engine now defaults to rolling strategy instead of recreate
- Improved error messages for failed health checks

### Fixed

- Race condition in canary promotion when multiple deploys overlap
- Redis connection leak in auth service during high traffic
- Sidebar collapse state not persisting across page reloads

---

## v2.0.0 - 2026-01-15

### Added

- Canary release strategy with configurable traffic weights
- Traffic splitting via service mesh integration
- `acme rollback` command with automatic snapshot restore

### Breaking Changes

- Deploy manifest format changed from v1 to v2. Run `acme migrate-manifest` to upgrade.
- Minimum Kubernetes version raised to 1.28

### Deprecated

- The `--recreate` flag on `acme deploy` is deprecated. Use `--strategy recreate` instead.

---

## v1.4.0 - 2025-11-20

### Added

- Distributed tracing with OpenTelemetry integration
- Trace viewer in the web dashboard
- `acme trace` CLI command for querying spans

### Fixed

- Service catalog returning stale entries after deregistration
- Deploy timeout not respecting custom values in manifest
