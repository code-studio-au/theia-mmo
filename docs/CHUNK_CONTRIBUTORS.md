# Geometry contributors for bounded chunk residency

Qualified bounded continuation, 2026-10-20. P1-03/P1-09/P1-10/P2-06/P2-07/P2-08,
execution-order row 5. Prerequisites: stable package identities, checked chunk
coordinates, bounded residency, paired readiness/publication and shared movement.
The qualified schema-1 fixtures and monolithic town remain regression baselines.

## Implemented contract

Schema 2 of `world.chunks.json` binds one conservative local XZ `bounds` envelope
to each package owner. The envelope includes its visual and collision geometry;
the package origin stays the existing checked 128-metre integer owner corner.
Index identity includes all bounds independently of entry order and query frame.
Schema 1 keeps its explicitly limited owner-only behavior and identity.

At index load, derive each indexed cell's contributor set from envelope overlap,
always including its own package. Bound index entries to 64 and contributors per
cell to the existing four-package residency cap. Conservative whole-package
bounds can retain extra geometry; they must never omit a crossing building.
Movement coverage includes a cell only when all its contributors are resident
and, for a network actor, reported ready. Input/gravity boundaries, demand,
protected actors, recovery and presented/interpolated roots use that same closure.
No file I/O or geometry discovery belongs in the simulation step.

Runtime collision preparation rejects actual collider bounds outside the indexed
envelope without changing active residency. Normal clients validate actual Bevy
mesh bounds through child transforms, independently of the render-root placement,
before initial readiness or replacement withdrawal/publication. Cached mesh
bounds remain usable after CPU vertex data is extracted for upload. Animated,
skinned and morphing chunk geometry is rejected in this static path; hierarchy
validation is bounded to 65,536 nodes and depth 128.

The offline `tools/index_chunk_contributors.py DIRECTORY NEW_INDEX` derives
combined bounds from real GLB scene transforms and all collision contributors,
verifies immutable package bindings twice and writes only a new index. It accepts
unextended static embedded GLBs; it rejects animated/skinned/morphing data and
external images. Outward 1/1024-metre rounding keeps exact 0/128 seams intact.
Node names are made unique in scratch data because the pinned Trimesh importer
can lose a transform when a node is named `world`; source assets are untouched.
Actual positions rather than accessor extrema determine visual extents.

Nondegenerate envelope intervals contribute on positive cell overlap; zero-width
planes on a seam contribute on both sides. Full capsule footprints still require
both adjacent complete cells before crossing. Readiness IDs name loaded packages;
complete movement cells are derived from those IDs and the index contributors.
Schema-1 identity/behavior remains unchanged for the qualified old experiments.
Wire/ALPN remains 10: the schema-2 dataset identity prevents mismatched admission.

These conservative combined envelopes establish contributors, not equality of
visual/collision meshes or source-model collision fidelity. Actual imported
partitioning, material/texture sharing and measured budgets remain before default
map adoption. The new original native cases pass publication with a lagging peer
and independent frames, valid-GLB extent rejection/retention, and valid initial
GLB rejection before readiness. The final workspace, authoring and retained native regression qualification
passes: 144 Rust tests, 51 Python tests and 15 actual two-client native cases,
plus both initial failure checks. See [status](STATUS.md) and
[executed evidence](VALIDATION.md).

## Acceptance

Use original geometry extending across a straight seam and a diagonal cell
corner. Approach from both sides; missing contributors must hold movement even
when the cell's own floor is present. Matching contributors must restore actual
wall/support queries. Reject underspecified bounds and over-budget contributor
sets without publication, preserve full footprints/recovery/presentation, and
bind extent changes into admission identity. Cover signed distant/query-frame
changes and retain existing schema-1 map and paired-handover regressions. Native
Bevy evidence is required for visual/publication claims. No production streaming,
whole task or phase is complete with this bounded contract.

## Measured next prerequisite: shared images

Read-only inventory of the prepared town (`build/chunk-contributors-town-partition-inventory.json`):
the GLB is 704,123,496 bytes, with 698,655,860 bytes in 491 embedded image buffer
views (about 99.2%). It has 4,266 nodes, 756 meshes/primitives, no animations or
skins, and no extensions. The collision profile contains 65,871 triangles and
3,972 conservative solids over roughly 533 by 533 metres. These are source-file
counts, not measured runtime memory or streaming capacity. All images are PNG;
header dimensions imply 1,524,977,664 base-level RGBA8 bytes, excluding mipmaps
and driver overhead. The largest image is 1024 by 2048; the largest compressed
image buffer is 2,637,448 bytes. These are format/header estimates, not measured
GPU residency.

Consequently the next imported-content slice must bind shared immutable image
resources into package/dataset identity and validate their paths/hashes before
using standard Bevy external-image loading. Do not copy the embedded image set
into every chunk. Then partition real terrain geometry at 128-metre boundaries
with matching collision, preserve model instance transforms and cross-cell
contributors, and measure resident/staging bytes and load behavior before default
adoption. Wrapping the entire town in one permanent contributor is not terrain
partitioning. Existing recovery anchors and the bounded four-package policy may
also require a measured refinement; this proof does not establish them as MMO
limits. Native reversal/supersession/asymmetric failures remain separate checks.
