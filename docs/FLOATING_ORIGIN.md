# Client render and physics rebasing

Chunk-world continuation, 2026-10-20: [normal chunk startup](CHUNK_STARTUP.md)
keeps a bounded common frame when following one actor would exclude another
loaded actor, static anchor or previous presentation endpoint. Distant joins and
recovery cannot force unchecked wire/render projections. Native original-scene
checks include actors 5.1 km apart; this is not region ownership or a capacity
benchmark. Earlier sections record the preceding bounded fixture contracts.


2026-10-19, bounded P1-03/09 and P2-05/07 continuation. Connected clients now
recentre their local render frame and actual Avian collider positions together.
Global authority, source content placements and wire v6 remain unchanged. This
extends [runtime coordinates and identity](COORDINATES_IDENTITY.md).

## Coordinate ownership

A client initializes an immutable content anchor from its reliable welcome. It
tracks its own server-owned pawn in global f64 coordinates. When that pawn exceeds
a local span on any axis, it selects a new snapped render origin using f64
subtraction/flooring relative to the content anchor. The default span is 128 m.
`THEIA_RENDER_SPAN=2` is a qualification override; valid spans are 1–1024 m.
Replication and render origins are independent: the server follows its oldest
actor for packet framing, while each client follows its own pawn for presentation.

Only marked static roots are reprojected: terrain/GLB roots, graybox scenery,
reference scenery and the actual Avian static collider entities. Each keeps an
immutable content-space translation, so placement derives from canonical data
rather than accumulated f32 shifts. Authored rotations/scales and child-local
skeletal/mesh transforms remain intact. Late scene children inherit the current
root transform. UI entities are not part of the rebased world.

Avian bodies move independently of parent transforms. Each collider's `Position`
and `Transform` therefore receive the same reprojected position directly. Standard
Avian plugins update their collider trees before that frame is captured. This is
an actual physics-world update, not a moved visual parent or a new physics engine.
Characters remain nonblocking query/presentation actors, without rigid bodies.

The local pawn's previous authoritative global pose and remote interpolation
endpoints remain global. On an origin change, their presentation is projected
before computing movement speed/facing. Remote interpolation runs in the current
frame from global endpoints; a frame shift does not become fake locomotion.

## Same-frame ordering

1. Update selects the client origin and presents server poses.
2. Static root placement follows presentation and the terrain/reference loading
   systems, including deferred root creation.
3. Standard Avian physics runs in PostUpdate, after these changes. This client
   world currently contains static bodies only; its scheduling does not set the
   authoritative server's fixed timestep.
4. Camera follow converts its local query points back to the immutable content
   collision frame. The existing shared Avian/profile clearance policy is retained.
5. Bevy propagates transforms, including imported scene hierarchies. Static mesh
   camera refinement then sees the new world transforms. The root camera's final
   global transform is synchronized before rendering extraction.
6. A Last-stage probe raycasts the actual client Avian BVH below the grounded
   local pawn, before capture. It requires ground alignment within 2 mm and records
   frame revision, probe count and maximum observed error.

The camera's content-space query cache remains immutable, separate from the
rebased plugin world. Coordinate conversion at that boundary is explicit. Server
movement also continues to query its fixed, bounded content/Avian frame. Neither
cache is silently moved along with client visuals.

## Verification and limits

Tests cover signed snap seams at distant origins, canonical landmark reprojection,
actual Avian collider-tree queries after a single update, all collider Positions,
unchanged child-local animation poses and late-child propagation. Interpolation
preserves its global path across rebases. Camera obstruction/visibility tests
verify identical clearance before/after a shift and recovery after orbiting.

Native checks use real servers/two Bevy clients at positive/negative billion-metre
anchors, repeated 2 m render spans, building access and the retained Quinn path.
Reports include both wire and render origins/counts, current physics frame revision,
actual ground-query counts/error and global/render pose checks. Exact results,
commands, devices and limitations are in [VALIDATION](VALIDATION.md).

This delivers automatic **client** render/static-physics rebasing for the bounded
fixture. It does not stream chunks, assign region ownership, rebase server query
scenes, or qualify dynamic-body contact/velocity/history rebasing. The client
continues to require admission into one bounded content frame (±4096 m per axis),
and the supplied terrain/profile bounds are smaller. General movement, prediction,
platforms, streaming and multi-chunk ownership retain their own acceptance gates.
No MMO capacity, performance or native Windows execution claim is made.

## Zero-delta startup/rebase continuation — 2026-10-20

An explicit schema-3 spawn away from the old origin exposed a first-frame query
mismatch: Avian skips its physics schedule at zero delta while presentation can
already admit/rebase an actor. The normal client now runs Avian's public
`update_moved_collider_aabbs::<Collider>` after physics preparation and before the
step when the content-to-render offset changes (including initialization). It
uses the existing standard trees; it does not advance gameplay time or introduce
another collision implementation. Ordinary unchanged frames avoid this extra
refresh. A regression verifies real ground/wall queries on the first zero-delta
update and a subsequent zero-delta rebase. The detailed probe assertion retains
both frame and content coordinates if another mismatch occurs.
