# Shared immutable world images

Qualified bounded continuation, 2026-10-20. P1-10/P2-06/P2-08, execution-order row 5.
Prerequisites: immutable package binding, geometry contributors and normal paired
Bevy/Avian publication. Town inventory shows 698.7 MB of its 704.1 MB GLB is
embedded images; real partitioning needs shared resources first.

## Implemented contract

World manifest schema 2 adds a canonical set of shared PNG descriptors: SHA-256,
encoded byte count, `color_space` (`srgb` or `linear`) and glTF sampler settings
`[wrapS,wrapT,minFilter,magFilter]`. Missing filters use zero; wrapping defaults
to repeat (10497). Package identity binds the complete set independently of JSON
order. Schema-1 identities remain unchanged and embedded fixtures remain valid.

Only fixed relative image URIs are allowed:
`../shared/SHA256-s-WRAPS-WRAPT-MIN-MAG.png` (or `-l-` for linear). Geometry stays
in the package's embedded GLB buffer. The URI's variant must match actual material
usage/sampler settings; arbitrary paths, URLs, aliases, external buffers and
unbound dependencies are rejected. Variant paths preserve Bevy's normal image
settings and share handles only for compatible uses. All files live inside the
explicit world asset root; no custom image loader or renderer is introduced.

Bounds: 512 unique descriptors, 16 MiB per encoded PNG, 1 GiB total encoded bytes,
16,777,216 pixels per image and 2 GiB estimated base-level RGBA8 bytes per package.
Manifest JSON is bounded to 256 KiB; GLB JSON to 16 MiB. These are initial content
validation bounds, not measured MMO residency capacity. Path/hash/resource checks
run before client asset loading on the existing preparation path; the headless
server still needs only collision data and the bound manifest identity.

`tools/externalize_world_images.py SOURCE DESTINATION --content-id ID --revision N`
preserves source packages, extracts original PNG bytes into shared immutable
files, removes their embedded views from the new GLB, retains geometry buffers,
node transforms and material values, and publishes a new collision/manifest
binding. Ambiguous texture indices used for both colour and linear channels are
split so the standard loader receives the right image variant. Same-ID changes
require a newer revision. Existing output packages are never overwritten; failed
extraction does not publish a package. Unreferenced immutable shared files may
remain after a failure. Identical PNG bytes in different variants may use hard
links, while the URI keys still give distinct Bevy assets when settings differ.
Different texture variants receive different paths; one compatible image key
across packages uses the same Bevy asset. Both declaration and actual GLB image
references must agree, including missing/unused/duplicate descriptor rejection.

## Acceptance

Cross-language canonical identity vectors; missing/corrupt/unsafe resource and
budget rejection before publication; compatible handle sharing and differing
colour-space/sampler variants; unchanged source geometry and image bytes; actual
Bevy textured roots during delayed publication and failed replacement retention.
Retain all qualified contributor, handover and normal map regressions. Measure
the staged town image extraction without promoting it as terrain partitioning or
runtime memory evidence. Actual terrain/collision partitioning, material budgets,
Windows and production streaming remain open.

## Identity and loading details

Schema 2 uses domain `theia-world-package-v2\0`, followed by the same logical ID,
revision and collision/visual byte/hash fields as schema 1. Append a little-endian
u16 resource count, then each canonical resource as SHA-256 (32 bytes), encoded
byte count (LE u64), colour-space byte (0 sRGB, 1 linear), and four LE u32 sampler
values. Sort by SHA bytes, colour-space byte and sampler tuple; duplicate variant
keys are rejected even when their declared byte counts differ. Resource order and
JSON formatting do not change identity. Shader dependencies must match actual
GLB image URIs and texture usage, not only the manifest declaration.

PNG headers must describe bounded 8-bit images. The actual decode and GPU upload
remain standard Bevy dependency loading; a valid hash alone is not readiness.
Prepared chunk roots use `AssetMetaCheck::Never`: unbound editor `.meta` files
cannot replace the bound loader settings. Native fixtures include deliberately
invalid PNG metadata files to verify this path. Ordinary embedded monolithic
launcher settings remain unchanged. Managed schema-1 real GLBs cannot load
unbound external images/buffers; existing embedded identities and qualified
binding-only headless fixtures remain intact.

Native capture inspects actual scene `StandardMaterial` image handles, resolved
asset paths, image formats/dimensions and sampler settings. Compatible paths
share one handle across real roots; colour-space or sampler variants keep
separate handles. It does not infer GPU-memory usage from these handles.

## Actual town staging

The unchanged prepared town was extracted to ignored
`build/shared-images-town-stage/town`, revision 3. Its GLB is 5,476,980 bytes and
488 unique shared resources contain the original 698,655,860 PNG bytes. The 491
source texture entries remain; aliased image entries share canonical resources.
This changes data placement, not total image compression or terrain partitioning.

Independent comparison verifies all 2,399 geometry accessor payloads and node,
mesh, material, sampler, scene and collision semantics against the actual source.
Every texture still references its original PNG bytes. Rust verifies the staged
manifest/collision/GLB and all 488 resource files. Unique PNG headers imply
1,514,491,904 base-level RGBA8 bytes, excluding mipmaps/driver overhead; this is a
header estimate, not measured GPU residency. Evidence lives in
`build/shared-images-town-extraction.log`,
`build/shared-images-town-byte-comparison.json` and the current private validation
log referenced in [validation](VALIDATION.md). The default town remains revision
2 and monolithic.

The asset graph must now be partitioned and pruned along with real terrain and
matching collision. Keeping unused textures/materials in every glTF would make
Bevy load dependencies unrelated to the selected cell, even with shared images.
Measure actual resident/staging/load behavior before default adoption. Complete
reversal/supersession/asymmetric error qualification and native Windows remain
open. No whole task, phase or MMO streaming capacity is established here.

## Qualified checkpoint

Fresh fmt, denied-warning Clippy/build, 151 default Rust tests, 56 Python tests,
18 actual two-client native cases and both retained startup failures pass. Both
ignored private-data cases pass separately: actual town resource verification
and both house routes at 20/30/60 Hz without recovery. Native image cases inspect
actual material handles/formats/dimensions/samplers during delayed publication,
compatible reuse, incompatible variants and corrupt-new-dependency retention.
All final native reports bind the same binaries/Cargo.lock on RX 7900 XTX/RADV
Vulkan with active validation. See [executed evidence](VALIDATION.md).

A retained delayed-peer case exposed an unload-only intermediate authority offer
when one peer moved within current coverage before the leading peer requested a
neighbour. Deterministic signed-frame and actual encrypted UDP tests now require
spare residency to remain until a new load needs the budget. Full superseding/
reversal qualification remains separate; the fix avoids this unnecessary churn
without changing explicit unload APIs or weakening readiness barriers.
