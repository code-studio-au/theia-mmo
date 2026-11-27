# Documentation review and cleanup

27 November 2026. Reviewed the repository's 235 Markdown files, including
first-party guides, asset documentation, templates and vendored provenance.
The structural audit checked local paths and section links across the collection;
current entry points, architecture, controls, networking and operational handoffs
received additional comparison against source and newer project records.
This is a documentation review, not fresh qualification of every recorded feature.

## Findings and changes

| Finding | Resolution |
| --- | --- |
| 201 files shared the top-level docs folder, mixing setup, contracts and old experiments | Categorised active documents by subject and added the [documentation index](../README.md) and folder indexes |
| Superseded handoffs still described old versions, pending releases and session permissions | Archived them with explicit historical scope; current operational entry points are the [launcher guide](../operations/GAME_LAUNCHER.md) and infrastructure runbooks |
| The engineering landing page and plans began with thousands of lines of checkpoint notes | Replaced the landing page with a reading guide and moved checkpoint histories into the [engineering archive](../archive/engineering/README.md) |
| The task catalogue repeated 109 inherited “NOT STARTED / NOT VERIFIED” placeholders | Removed those duplicate placeholders and linked actual implementation evidence; task IDs, prerequisites and acceptance criteria are retained, and P3-17/18 remain unspecified |
| Architecture still repeated hybrid combat defaults, the old project identity and unresolved dependency/provider choices | Reconciled the active document with adopted Classic controls, the actual workspace and documented identity providers; preserved the [original baseline](../archive/engineering/ARCHITECTURE_BASELINE.md) |
| Movement and formula guides presented earlier melee/facing gaps as current | Identified the old matrices as historical and linked the subsequent qualified slices |
| Networking guides mixed old shared-wire and Quinn ALPN identifiers | Added current source references: shared wire version 14 and retained Quinn ALPN `theia-mmo-walk-12` are separate identifiers |
| One Decisions link used the wrong generated heading for “raise/lower” | Corrected the section fragment |
| Moving quest guides would break sibling story links | Updated 11 story documents to link directly to the moved guides and removed the two obsolete forwarding pages |

## Retention and deletion

There are 53 historical records in [the archive](../archive/README.md), including
the extracted engineering histories and original architecture. Frozen forest
imports, superseded environment experiments, dated performance/asset checks and
session handoffs retain unique evidence or provenance. Active file-format
contracts, asset preparation, regression-map guides and gameplay documentation
remain in the subject folders even when their first qualification is old.

No exact duplicate standalone documents were found. No unique document was
permanently deleted merely because it was old. The two redundant quest forwarding
pages were deleted after their incoming story links were updated. Redundant task-status placeholders
were removed, and duplicate checkpoint prose was removed from active architecture
while its source remains archived. Licenses, third-party notices and README
templates used by generators remain beside their code or assets.

References were updated in Markdown, contributor instructions, source comments
and launcher error messages. Source-comment and error-message changes do not alter
gameplay or launcher control flow. At the user's subsequent request, 11 documents
in the sibling `theia-story` repository now reference the moved quest guides directly.

## Verification

- All first-party local Markdown paths and section links resolve after the move.
- Original prose was compared with relocated/archived text, ignoring rewritten
  document destinations. Intentional editorial changes were reviewed separately;
  task headings, prerequisites and acceptance criteria were retained.
- `cargo fmt --check` and `git diff --check` pass.
- Four local-launcher tests and five map-placement tests pass. The latter used
  the existing `build/asset-tools` Python environment; system Python lacks
  `trimesh`. Changed Python files also pass syntax checks.
- Workspace denied-warning Clippy remains blocked by missing ALSA and Wayland
  development libraries. No rendering behavior changed, so native GPU runs were
  not repeated.

## Remaining gaps

The vendored [Landmass README](../../third_party/landmass/README.md) links to
`LICENSE-APACHE` and `LICENSE-MIT`, but both files are absent from the vendored
directory and the available cached crate. Its [provenance note](../../third_party/landmass/THEIA.md)
says original licenses were retained. This pre-existing inconsistency needs
upstream provenance verification; no license text or replacement terms were
invented during this cleanup.

Historical guides still contain environment-specific `build/` evidence paths.
Their absence from a fresh checkout is expected and does not invalidate the
recorded run. External websites, published releases, live cloud state and every
historical command were not revalidated by this structural review. Dated
qualification remains evidence for its own revision and platform.
