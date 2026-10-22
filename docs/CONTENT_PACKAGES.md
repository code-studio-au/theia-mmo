# Versioned bounded world packages

P1-10/P2-08 continuation, 2026-10-19. This is the content identity prerequisite
for general collision and bounded chunk loading. The current tile remains one
monolithic visual/collision fixture; no streaming or general controller is added
by this slice. Renet, wire v6 and the default 30 Hz authority remain unchanged.

## Files and identity

A prepared directory can contain `world.manifest.json`, `terrain.json` and
`terrain.glb`. Manifest schema 1 has these required fields, with unknown fields
rejected:

| Field | Meaning |
|---|---|
| `schema` | Exactly 1 |
| `content_id` | Stable logical ASCII name, 1–96 characters |
| `revision` | Nonzero u32 immutable content revision |
| `collision_schema` | Existing height-surface fixture format, 1 or 2 |
| `collision_bytes`, `collision_sha256` | Exact profile length and 32 digest bytes |
| `visual_bytes`, `visual_sha256` | Exact GLB length and 32 digest bytes |

Logical names begin with a lowercase letter, end with a lowercase letter/digit,
and contain lowercase letters, digits, `.`, `_` or `-`; `..` is forbidden.
`theia.development.lordaeron-town` is a logical identity, independent of an export
folder or absolute file path. It is separate from the world incarnation epoch
and transient actor identities. New artifact bytes require a new revision and
prepared output directory. There is no global registry enforcing revision
uniqueness; the package fingerprint always includes actual artifact digests.

Schema 1 fixes artifact names; it accepts no arbitrary file paths. World geometry
is Y-up metres in the existing bounded local content frame. The server's admitted
global anchor is separate. Profile schema 1 retains its legacy patch/column rules;
schema 2 retains explicit bounds/support surfaces and finite boxes. A future
arbitrary-mesh collision format or chunk index requires an explicit format/schema
extension, not silently reinterpreting the height-surface format.

Budgets: manifest 8 KiB, collision profile 32 MiB (with existing schema-1's tighter
4 MiB decode limit), visual 2 GiB. Collision bytes are bounded during reading,
then verified against the manifest before geometry decoding. Its declared schema
and embedded visual digest must also agree. The headless server needs only the
manifest/profile; it never opens or decodes GLB content. The client additionally
checks visual length/digest using a streaming 64 KiB hash buffer, then uses normal
Bevy GLB loading. Hash verification no longer allocates a second entire visual
file; Bevy's own asset loading still has its normal memory cost.

The admitted 32-byte package fingerprint is SHA-256 of these concatenated bytes:

1. ASCII `theia-world-package-v1` followed by a zero byte.
2. Logical-name length as u16 little-endian, then its ASCII bytes.
3. Revision u32 LE, collision schema u32 LE, collision length u64 LE.
4. Collision SHA-256 (32 raw bytes).
5. Visual length u64 LE, then visual SHA-256 (32 raw bytes).

Manifest whitespace/key order and filesystem paths do not affect this identity.
Rust/Python share a checked golden fixture and fingerprint. Normal secure welcome
world matching therefore rejects different logical IDs, revisions or artifact
bindings, without changing the wire shape. Diagnostics report logical ID/revision
separately from the fingerprint.

If a manifest exists but is malformed, oversized or inconsistent, loading fails;
there is no fallback from a broken package. For retained historical fixtures,
standalone loaders still accept a directory with no manifest, using the old exact
profile-byte digest. A legacy-loaded client cannot match a packaged server even
with identical profile bytes. The normal `tools/play.py` path now requires the
manifest to prevent accidental use of an unpackaged development world. Packages
are local immutable inputs: hashes establish consistency, not publisher trust,
asset licensing or protection against filesystem changes after admission.

## Prepare and publish

Run existing preparation/access tools first, then publish the completed directory:

```bash
python3 tools/package_world.py build/development-world-town \
  --content-id theia.development.lordaeron-town --revision 1
python3 tools/play.py --clients 2
```

The publisher exclusively creates the manifest and refuses an existing one. It
hashes existing outputs without modifying profile/visual/source bytes, checks
size/schema/visual binding and prints the resulting fingerprint. It does not
validate arbitrary geometry or glTF; those existing runtime/import checks remain.
When changing collision/visual artifacts, prepare a new directory and publish
its next revision after the final edit. Do not run a mutating access tool against
an already published directory and expect the old manifest to remain valid.

The existing ignored outdoor, farmhouse, town and terrain-patch fixtures have
been packaged as revision 1 under separate logical names. The default town is
`theia.development.lordaeron-town`, fingerprint
`876c03c5f36ab2bbe1575ecd24897bee334504aa208e390cb168ddf824e454c1`, binding
12,204,375 collision bytes and 704,123,496 visual bytes. Source assets stay outside
Git; the checked-in manifest fixture is synthetic metadata only.

## Verification and next gate

`content_packages.rs` checks cross-language golden identity, formatting stability,
revision sensitivity, malformed/oversized manifests, altered profile/visual bytes,
profile/schema binding, fail-closed present manifests and legacy fixture behavior.
A real authenticated UDP authority admits the matching package and rejects a
client expecting the previous revision, with the server's visual file absent.
Python tests cover exclusive publication, unchanged inputs, path independence,
invalid identity/revision/binding, missing files and sparse oversized artifacts.
Native map/launcher qualification is recorded in [validation](VALIDATION.md).

Next: define chunk ownership from the existing checked `ChunkId` convention,
bounded residency/load/unload and visual/collision ownership, then qualify a
shared general kinematic controller with stacked floors, steps/slopes/headroom,
falling/jumping and nonblocking actors. Existing authored house routes remain
regressions. No entire P1-10/P2-08 task, phase or wave is closed by this package gate.

Collision schema 3 now supports general static mesh geometry and explicit spawn
metadata using the same manifest identity layout. See [current qualification](GENERAL_COLLISION.md);
native/default town revision-2 adoption now passes.

## Shared dependency continuation

[Manifest schema 2](SHARED_IMAGES.md) binds canonical shared PNG variants,
including immutable byte/hash, colour-space and sampler settings. The v2 domain
and resource byte layout are specified there; schema-1 identities remain intact.
The normal chunk client validates exact GLB URI/settings agreement, in-root paths,
PNG byte/header budgets and hashes before standard Bevy loading. Server collision
loading still needs no image I/O. The offline extraction tool publishes a new
representation without changing source geometry or the active map.
