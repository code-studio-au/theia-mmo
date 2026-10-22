# AGENTS.md — Theia repository instructions for Codex
Read `ARCHITECTURE.md`, `IMPLEMENTATION_PLAN.md`, and `CODEX_TASKS.md` before implementing features. Approved decisions are design constraints, not evidence of existing code. Verify the repository and toolchain first. Never claim a task is complete without tests and evidence.

## Fixed architectural constraints
Rust + Bevy, headless authoritative 30Hz simulation, Renet initially, Avian initially, f64 global/f32 local coordinate conversion, PostgreSQL async persistence, Bevy/wgpu rendering, modular monolith and logical world regions. Maintain data-driven gameplay and content. Modernized WoW visual style with GW2-inspired mobile casting. Use original or appropriately licensed production assets. Critical boss telegraphs and audio take precedence over cosmetic load.

## Workflow
1. Identify relevant Pn-nn task, dependencies, wave, and acceptance requirements.
2. Inspect actual code; classify present/missing/obsolete. Do not overwrite working experiments without review.
3. Create minimal tests first and preserve server/client separation.
4. Keep input queues bounded and avoid DB/network/render blocking the simulation tick.
5. Validate across region boundaries, packet loss, crowded scenes and failures where applicable.
6. Report exact files changed, commands run, tests passed/failed, measured metrics, and unresolved issues.
7. Update CODEX_TASKS.md status only after evidence; ask before changing locked architecture.
8. Treat P3-17/18 as unknown until owner supplies missing scope; do not invent historical approvals.
9. Do not treat preliminary wave assignments as strict serial gates; execute by actual dependencies.
10. Never commit secrets, proprietary game assets without authorization, or undocumented binary formats.

## Initial next task
Audit repository and map code to P1-01..P1-12, then prioritize fixed-timestep headless scheduler with bounded command queues (historically called M1-04). Record any task ID discrepancy.
