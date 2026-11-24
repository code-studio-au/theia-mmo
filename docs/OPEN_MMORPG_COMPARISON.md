# Theia and Open-MMORPG comparison — 11 November 2026

Reviewed Theia at `3db337eb908bc93dba7f7ba5c545f3aa2a14a106` and
[developerz-ai/open-mmorpg](https://github.com/developerz-ai/open-mmorpg) at
[`90fa3ca89f02fc82bd0322ad08a2e7a1181aef36`](https://github.com/developerz-ai/open-mmorpg/tree/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36).
This is a source and documentation review, not an independent run or benchmark
of Open-MMORPG. Historical Theia qualification refers to the exact recorded
binaries, fixtures and platforms. Neither project has established MMO-scale
production capacity.

## Assessment

Theia is ahead in an actual visual, connected gameplay and authoring slice.
Open-MMORPG has a broader operator-platform vision and more explicit web,
modding and distributed-service scaffolding. Its own [current status](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/README.md)
says the gateway-to-shard gameplay path is not yet wired; its
[client](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/apps/client/src/main.rs)
is a headless prediction example. Do not read its aspirational front-page
feature list as completed production capabilities.

| Area | Theia | Open-MMORPG |
| --- | --- | --- |
| Playable client | Bevy renderer and connected prototype with documented native fixtures. | Headless prediction demo; graphical client planned. |
| Multiplayer | Authoritative Rust server and secure Renet launcher path, with bounded real-UDP/two-client evidence. Quinn is a retained alternative. | Fixed-tick shard and separate transport components; [shard main](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/apps/shard/src/main.rs) currently drains no network input. |
| Combat/content | Classic-oriented targeting, melee, ability fixtures, NPCs, quest/inventory and durable slices, with different activation and qualification boundaries. | Simulation and ability/datapack foundations exist; client/server end-to-end gameplay remains a release target. |
| Authoring | Bevy zone editor, terrain and greybox workflow; actual Sprinkles particle authoring is embedded. Particle effects are not yet placed/exported into the world. | Plans a data-driven/MCP editor; a comparable visual editor is not established by this review. |
| Persistence/auth | PostgreSQL/OIDC and bounded, documented normal/opt-in gameplay flows. | SQLx account/character repositories and migrations; ownership reference is in-memory, and persistence is not yet a shard dependency. The gateway uses a development verifier. |
| Operator/web | No comparable operator portal established by this review. | Bun/SolidJS operator website, shared UI and i18n, plus gateway and world-service scaffolds. Some web paths have an in-memory mock backend; social/market service state is in memory. |
| Scale/streaming | f64 global positions, bounded f32 local frames, Avian and measured chunk/transport experiments; default world and distributed ownership are not production streaming. | Multi-shard autoscaling, handover and seamless zones are explicit architectural/release goals, not established current behavior. |
| Extensibility | Data-driven content, validated packages and dedicated editor. | Manifest/datapack schema, compiled module hooks, and Wasmtime scripting foundation; compiled modules require rebuilds. |

Theia's [local Renet evidence](RENET_QUALIFICATION.md) is bounded and should not
be extrapolated to an Internet service or MMO player counts. Its
[ability rules](CLASSIC_ABILITY_RULES.md), [zone editor](ZONE_EDITOR.md),
[particle integration](SPRINKLES_INTEGRATION.md), and [roadmap](ROADMAP.md)
separately document live slices, optional fixtures and remaining work. In
Open-MMORPG the [MCP companion](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/apps/mcp/src/main.rs)
is a bounded-action stub, not a running in-game companion system. Its
[ownership store](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/crates/persistence/src/lib.rs)
is an in-memory reference, despite the durable transaction design.

## Stack and roadmap

Theia pins Bevy `0.20.0`, an Avian development revision, Renet/netcode 2.0 and
Quinn as a retained baseline; Rust runs the client and authoritative server,
PostgreSQL backs qualified persistence slices, and Linux is primary with Windows
first-class but incompletely verified. Its current roadmap builds bounded playable
slices and tests real integration before broader deployment and region ownership.

Open-MMORPG's [workspace](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/Cargo.toml)
pins Bevy subcrates 0.19, Tokio/Axum, SQLx and Wasmtime; the web workspace uses
Bun, SolidJS and Vite. Its [v1 plan](https://github.com/developerz-ai/open-mmorpg/blob/90fa3ca89f02fc82bd0322ad08a2e7a1181aef36/docs/mvp/v1/README.md)
targets web login, a graphical client, gateway, authoritative shard, persistence,
a starting zone, dungeon, auction house and a two-shard handover release gate.
Yugabyte and horizontal scaling are targets, not proof of a live scalable realm.
The current simulation uses plain component structs and f32 positions; cross-
platform bit-identical replay should be demonstrated before relying on the claim.

## Strengths, weaknesses and useful lessons

- **Theia strengths:** real rendered/editor workflows, integrated multiplayer and
  database-qualified vertical slices, strong evidence boundaries, large-world
  precision and current Sprinkles reuse. **Weaknesses:** many gameplay pieces are
  optional or narrowly qualified, documentation/checkpoints are sprawling, particle
  world placement is unfinished, and native Windows, operational tooling,
  streaming, scale and production security remain open.
- **Open-MMORPG strengths:** clear public MIT repository, operator web/UI and i18n,
  documented extension contracts and an explicit end-to-end release gate.
  **Weaknesses:** substantial integration remains before a rendered playable
  realm; gateway verifier, social/auction projections and item ownership have
  development/in-memory limits; sharding, companions and full renderer are plans.
  Its large service and release scope adds integration/operations risk.
- **Recommendations:** keep Theia's selected Bevy/Avian/Renet architecture and
  finish a small end-to-end playable slice. Borrow the *ideas* of an explicit
  release gate, versioned content/module contracts, operator UX and i18n. Review
  specific MIT components only if a concrete requirement justifies the dependency,
  with provenance, compatibility and tests; do not import its second Bevy version,
  custom networking or distributed topology wholesale. Benchmark both only with
  matched workloads before making performance or scalability claims.
