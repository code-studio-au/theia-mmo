# Theia Online — Hosting and Operations Requirements

Status: agreed direction, 24 November 2026. Detailed implementation design pending review of each phase. This document specifies requirements, not completed infrastructure.

## Scope and constraints
- Initial private development environment for two testers; use existing AWS account.
- Sydney (ap-southeast-2), EC2 t3a.medium (x86-64, 2 vCPU, 4 GiB). Benchmark sustained simulation, memory and CPU credits before expanding.
- Proposed Ubuntu 24.04 LTS, 40 GiB gp3 EBS, static public IPv4, dedicated network/security group, systemd service and Systems Manager administration. No public SSH by default.
- Single instance for game server, authentication and persistence initially; confirm actual database implementation and runtime dependencies from source. No Kubernetes, load balancer or dedicated database initially.
- GitHub Pages is the selected static launcher/client distribution channel; retain existing package discovery/update behavior. Validate Pages size, bandwidth and usage constraints and asset redistribution rights; use release assets for large packages if needed. Do not publish secrets or private assets.
- Local launcher changes may not yet be committed. Audit current main and reconcile local changes before implementing. Existing Rust/Bevy workspace, Renet networking and auth/persistence crates are starting points; do not assume README reflects current implementation.

## Base architecture and long-term scaling (creator-approved)
- Adopt a modular Rust monolith initially on one EC2 instance. Distributed readiness is a formal requirement, not a stretch goal; defer actual distributed deployment until justified by measured demand.
- Establish typed interfaces and explicit state ownership for authentication/login/session and character selection, world coordination, authoritative simulation, social services (chat, friends, guilds, parties), economy (trade, auction house, mail), persistence and administration. Avoid cross-module mutation and process-local assumptions in future service contracts. Keep the independently available gateway/reporting architecture.
- Rust remains the default backend and orchestration language; Go is not required. Luau is the gameplay scripting direction through validated Rust APIs; systemd manages processes, Terraform provisions infrastructure and GitHub Actions deploys.
- Establish stable realm, zone, layer, instance, character and server-process identities; distinguish logical ownership from physical addresses. Preserve existing coordinate and authority contracts. Define routing and handoff interfaces with local implementations initially; future transfers require authenticated destination readiness, ownership fencing, retry/recovery and reconnect semantics. Never permit simultaneous authoritative ownership of a character. Persistence must survive process replacement.
- Economy and inventory operations must be atomic, durable, auditable and idempotent, preventing item duplication or loss across retries and crashes. No separate database per module initially.
- Long-term: independently extract login, chat/social, economy/auction house, persistence and world coordination services when capacity or availability warrants. Define cross-zone/layer social and economy scope before extraction.
- World coordinator manages player allocation, zone ownership, population layers and private instances where game design permits; preserve party cohesion, PvP rules and world continuity. Infrastructure controller manages host/process capacity and health. Generic load balancing does not replace game-aware routing.
- Scale from multiple processes on one host to multiple hosts after profiling and load tests. Collect per-process/zone population, tick latency, CPU/memory, network and persistence metrics. Define capacity thresholds, hysteresis, draining and recovery before automatic scaling.
- Phase 1 design gate: audit existing module boundaries, identity/authority and persistence contracts; introduce only foundations needed now. Later distributed implementation requires explicit design review and tests for handoff, recovery, duplicate prevention and cross-service behavior.

## Phase 1 — Foundation and manual operations
- Version-controlled infrastructure as code (Terraform proposed under infrastructure/aws), with protected state, least-privilege roles, resource tags and budget alerts. Review plan before provisioning paid resources.
- Reproducible, locked builds for Linux server and supported client/launcher platforms; reuse existing scripts and asset preparation. Build on CI runners, not EC2. Ensure server package includes required world/collision data and can run headless.
- Separate CI validation, server deployment, client publishing and launcher publishing workflows under .github/workflows. Manual release/deploy triggers initially; commits must not restart the server. Pin dependencies/actions, use GitHub OIDC and scoped AWS roles, no long-lived AWS keys in repository or client.
- Immutable versioned artifacts, manifests, integrity checks (SHA-256 and signed release metadata), protocol/content compatibility checks, atomic installation and rollback. Publish manifests only after packages are available. Preserve compatible server/client release sets.
- Deploy via Systems Manager or equivalent authenticated mechanism; graceful drain, backup before incompatible migrations, health checks and rollback to last known good release. Database rollback must respect schema compatibility.
- Verify remote networking, bind/advertised addresses, secure authentication, certificate/token handling and persistence before exposing the server. Restrict network access to required ports and testers where practical. No public database.
- Manual start/stop/restart via AWS console initially. systemd starts game service on boot and handles unexpected process failure. Persist data across EC2 stop/start; encrypted storage, backups and tested restore.
- First-class observability from first remote test: structured Rust tracing, CloudWatch logs/metrics, bounded retention, CPU credits, memory/disk (agent or custom metrics), network, tick duration/rate, entity/player counts, latency, readiness, database connectivity and backup status. Include timestamps, build IDs and non-secret correlation/session IDs shared across client and server. Never log credentials or bearer tokens.

## Phase 2 — On-demand lifecycle and reporting
- Independently available Theia Gateway using API Gateway and Lambda. Authenticated player wake/status requests; admin permissions separate. Rate limit and make wake requests idempotent. No gameplay traffic through gateway.
- Launcher Play requests wake automatically if stopped, polls status and launches client only after application readiness. States: offline, starting, ready, draining/shutting down, failed; distinguish EC2 running from game/database readiness. Return validated connection endpoint. Provide timeout/retry and useful errors.
- Zero connected players triggers proposed five-minute idle grace period. Pending joins/reconnect reservations prevent shutdown. Serialize wake/drain transitions, handle concurrent requests and boot failure; save/flush persistent data before stopping EC2 (not merely exiting the game process). Provide maintenance override and manual fallback. Avoid stop/start loops. Compute stops billing while stopped; storage, IPv4 and other retained resources may still cost money.
- Collect server and client errors/crashes, including launcher failures where practical. Reporting endpoint remains available while EC2 is offline. Local bounded queue for offline submissions; retry with deduplication and rate/size limits.
- In-game Report Bug: description, optional screenshot with consent, zone/location, relevant diagnostic state, recent sanitized logs, client/server versions, correlation ID and timestamp. Preview/redact sensitive data; exclude credentials, tokens and unrelated personal data. Crash diagnostics best effort and consent-aware.
- Proposed DynamoDB report metadata and private S3 attachments, with scoped access, encryption, retention/deletion and upload validation. CloudWatch for operational telemetry. Define schemas, quotas and cost limits before implementation.

## Phase 3 — Administration panel
- Separate static web UI (GitHub Pages proposed), available when game instance is stopped. Actual authorization enforced by authenticated API, not by hiding the static page. No embedded privileged credentials. Role-based access, admin audit trail, confirmation for destructive actions.
- Infrastructure: EC2 lifecycle, CPU/credits, memory, disk, network, uptime, running hours and AWS costs (billing data may lag).
- Game: readiness, connected players, tick performance, entities, latency, build and database health/last backup. Clearly label stale/unavailable metrics when stopped.
- Controls: start, graceful stop, restart, maintenance override; deployment/version history, approved release selection and rollback.
- Errors: searchable server/client/launcher events, crash grouping, severity, correlation, recent logs and trends.
- Bug reports: searchable queue, attachments, status, priority, investigation notes, optional GitHub Issues integration with explicit action and sanitized content.
- Use shared gateway/API contracts with separate player and administrator authorization. Admin operations must not expose AWS credentials to browser.

## Acceptance gates and open design decisions
1. Phase 1: reproducible CI artifacts; manual deployment and rollback; two remote clients connect securely; data survives restart; backup restore and logs/metrics verified.
2. Phase 2: launcher wakes stopped instance; readiness is accurate; concurrent joins and reconnects prevent premature stop; idle stop preserves data; offline/error/bug reports arrive without game server running.
3. Phase 3: authorized admin can inspect health, manage lifecycle and releases, correlate errors and triage reports; unauthorized users cannot perform privileged operations.
- Review exact ports, database engine, authentication, release manifest schema, asset packaging, Terraform state/bootstrap, DNS/TLS, IAM, idle policy, telemetry schema/retention, budget and dashboard technology during detailed phase design.
- No AWS resources or workflows are provisioned by this requirements document. Implement only after phase-specific review.
