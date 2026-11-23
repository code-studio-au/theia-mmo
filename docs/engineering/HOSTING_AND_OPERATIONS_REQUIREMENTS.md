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

## Separation of concerns — engineering policy (creator-approved)
- Prioritize reviewing the existing networking crate's simulation responsibilities. Progressively move authoritative world state, physics, combat and NPC behavior behind a simulation boundary; networking owns connections, transport, protocol encoding and replication delivery. Simulation owns authoritative outcomes and exposes typed commands/events/snapshots; shared contracts remain transport-independent. Avoid circular dependencies.
- Keep gameplay independent of AWS and deployment APIs; infrastructure integrates through adapters. Authentication remains independent of gameplay, with explicit session/admission contracts. Persistence uses asynchronous bounded workers and must not block the simulation tick; preserve durable acknowledgement and ownership guarantees.
- New chat, guild, party, trade and auction-house features must define state ownership, interfaces and transaction/consistency requirements before implementation. Luau scripts use controlled, validated gameplay APIs rather than direct database, transport or infrastructure access.
- Preserve existing core, auth and persistence boundaries. Audit dependencies before extracting a simulation crate; use incremental, tested changes rather than a wholesale rewrite. Maintain existing protocol compatibility where required, gameplay behavior and regression tests. Document exceptions and migration steps.
- Treat separation as an ongoing design/review criterion, not a requirement to deploy separate processes now. Keep the initial single-server deployment and avoid delaying gameplay for speculative refactoring.

## Phase 1 — Foundation and manual operations
- Version-controlled infrastructure as code (Terraform proposed under infrastructure/aws), with protected state, least-privilege roles, resource tags and budget alerts. Review plan before provisioning paid resources.
- Reproducible, locked builds for Linux server and supported client/launcher platforms; reuse existing scripts and asset preparation. Build on CI runners, not EC2. Ensure server package includes required world/collision data and can run headless.
- Separate CI validation, server deployment, client publishing and launcher publishing workflows under .github/workflows. Manual release/deploy triggers initially; commits must not restart the server. Pin dependencies/actions, use GitHub OIDC and scoped AWS roles, no long-lived AWS keys in repository or client.
- Immutable versioned artifacts, manifests, integrity checks (SHA-256 and signed release metadata), protocol/content compatibility checks, atomic installation and rollback. Publish manifests only after packages are available. Preserve compatible server/client release sets.
- Deploy via Systems Manager or equivalent authenticated mechanism; graceful drain, backup before incompatible migrations, health checks and rollback to last known good release. Database rollback must respect schema compatibility.
- Verify remote networking, bind/advertised addresses, secure authentication, certificate/token handling and persistence before exposing the server. Restrict network access to required ports and testers where practical. No public database.
- Manual start/stop/restart via AWS console initially. systemd starts game service on boot and handles unexpected process failure. Persist data across EC2 stop/start; encrypted storage, backups and tested restore.
- First-class observability from first remote test: structured Rust tracing, CloudWatch logs/metrics, bounded retention, CPU credits, memory/disk (agent or custom metrics), network, tick duration/rate, entity/player counts, latency, readiness, database connectivity and backup status. Include timestamps, build IDs and non-secret correlation/session IDs shared across client and server. Never log credentials or bearer tokens.

## Phase 1 platform and modular release packaging (creator-approved)
- Linux only for server, client and launcher initially; generic x86-64, Ubuntu 24.04 LTS build/server baseline, Linux Mint 22 on AMD CPU/GPU as primary client qualification platform. Pin existing Rust toolchain and dependencies; validate Vulkan/Mesa and runtime libraries. Headless server must not require desktop/GPU.
- Independently version server, client executable and launcher releases. Preserve launcher self-update behavior; audit current local implementation before changes. Prefer compressed user-installable archives initially.
- Replace whole-client updates with manifest-based package-level patching in Phase 1. Independently version client executable, UI, character/equipment/animation assets, world objects, terrain by zone/region, audio and client-facing gameplay content. Final package boundaries follow actual dependencies and asset layout; keep authoritative server-only data private.
- GitHub Pages hosts the signed release manifest and package distribution subject to hosting limits and redistribution rights. Manifest specifies release/protocol compatibility, package IDs, versions, dependencies, sizes, URLs and SHA-256 hashes. Use immutable package URLs; publish manifest last. Large artifacts may use release assets where needed.
- Launcher compares installed package inventory with approved manifest, downloads only missing/changed packages, verifies signatures and hashes, stages and validates a complete compatible release before atomic activation. Preserve previous playable release on interruption/failure; handle removed packages, disk space, concurrent launch/update and rollback. No administrator privileges for routine updates.
- Package-level replacement first; binary delta patching deferred until justified. Acceptance: clean install, executable-only update, single terrain-package update, launcher self-update, corrupted/interrupted download rejection and rollback, compatible remote multiplayer on Mint 22. No unrelated assets should be downloaded for an isolated package change.

## Release versioning and channels (creator-approved)
- Development is the active channel and default for initial testing; Stable is reserved for explicitly approved known-good releases and may remain empty for an extended period. Stable does not imply production readiness.
- Independently version launcher, client, server and asset packages; record immutable build identifiers and Git commit SHA. Track network protocol and content/schema compatibility separately. A signed release manifest identifies the exact compatible artifact set.
- Maintain separate channel manifests and launcher channel selection; handle unavailable Stable gracefully. Promote the exact tested immutable artifacts from Development to Stable without rebuilding. Manual publishing, promotion and server deployment via GitHub Actions initially.
- One hosted game environment and one active server release initially; two channels do not imply two live worlds. Validate client/server compatibility before Play and clearly report mismatches. Preserve prior release artifacts for rollback, subject to database/schema compatibility.
- No requirement to publish a Stable release during Phase 1; qualify the promotion mechanism without exposing unapproved builds as Stable.

## CI/CD trigger policy (creator-approved)
- Automatic CI validation on pushes and pull requests; automatic release-candidate builds on pushes to main. Detect affected server, client, launcher and asset packages using dependency-aware change detection, including shared crates, lockfiles, toolchain and build configuration; fall back to full builds when uncertain.
- Separate reusable server, client and launcher build workflows, CI validation, manual publish and manual AWS deploy workflows; provide a manually triggered combined release workflow. Build candidates are immutable and traceable to commit, with compatible artifact sets and retained rollback candidates.
- Publishing to GitHub Pages, Stable promotion and AWS deployment remain explicitly manual. No successful build or push may automatically change a live release or restart the server. Failed or incomplete builds cannot publish. Serialize publication/deployment, cancel superseded unneeded builds safely, and verify compatibility and health before activation.
- Future idle-window automatic Development publishing/deployment is deferred pending explicit approval and lifecycle safeguards.

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
