# Linux launcher and Pages handoff · 25 November 2026

Safe Pages publishing and optional native Cognito sign-in are merged into
`theia-mmo/main`. The website download link and automatic news import are in
the website draft. The public release still needs the CI build and
clean-machine qualification. Native sign-in and automatic
Sydney wake are separate AWS rollouts.

## Checkouts

| Repo | Branch to fetch | Contents |
| --- | --- | --- |
| [code-studio-au/theia-mmo](https://github.com/code-studio-au/theia-mmo/tree/main) | `main` | Release candidate build, safe Pages publishing and signed release news merged in [PR #58](https://github.com/code-studio-au/theia-mmo/pull/58) as `d066546b`. Native sign-in merged in [PR #59](https://github.com/code-studio-au/theia-mmo/pull/59) as `ae89892a`. |
| [code-studio-au/theia-website](https://github.com/code-studio-au/theia-website/tree/development/verified-linux-download-link) | `development/verified-linux-download-link` | Website opt-in, pre-deploy checks, stable Linux link and signed news import. Latest pushed implementation `dcd5583`, based on `main` at `25c61f6d`. |

The [website PR #1](https://github.com/code-studio-au/theia-website/pull/1) is open as a draft.
Check the latest remote state and preserve any local work before switching
branches. Read [the game launcher runbook](GAME_LAUNCHER.md) and
[the website deployment notes](https://github.com/code-studio-au/theia-website/blob/development/verified-linux-download-link/README.md).

## What is live

- `https://theiaonline.com/download/` is the website project on GitHub Pages.
  The Linux download is currently unavailable there. The live page was checked
  and had no Debian package link.
- [The site's RSS feed](https://theiaonline.com/rss.xml) is already live.
  It still returned HTTP 200 after the game merges.
- The launcher expects
  `https://code-studio-au.github.io/theia-mmo/downloads/development.json`.
  The feed still returned HTTP 404 before publication. The release workflow
  is manually dispatched and reads the reviewed asset list from `main`.
  Game Pages now uses GitHub Actions. The `game-downloads-release` environment
  requires the repository owner's approval and accepts `main`. The Ed25519
  secret is stored in that environment with a private local backup. The
  website release-news trigger credential remains pending.
- Sydney Cognito sign-in and native Millhaven entry previously passed for one
  Linux tester using launcher `0.1.6` and client `0.1.0.12`. See
  [STATUS.md](STATUS.md). This is evidence for the manually configured tester
  package, not for a download from the public site.
- Gateway and scoped AWS roles are merged in
  [PR #56](https://github.com/code-studio-au/theia-mmo/pull/56). Binary patches
  are merged in [PR #57](https://github.com/code-studio-au/theia-mmo/pull/57).
  Full GitHub Rust CI passed on their combined `main` revision
  [`a0ae08d4`](https://github.com/code-studio-au/theia-mmo/actions/runs/45540097315).
  No AWS gateway deployment was dispatched.

## Native Sydney sign-in

Launcher `0.1.9` includes a GTK sign-in dialog and the packaged Cognito SRP
helper when built by the Pages workflow. The game client still uses Bevy for
the in-game UI, including the character roster. The launcher dialog supports
temporary password changes and SMS or authenticator codes, and offers managed
browser sign-in. Existing realm configurations continue using browser login.
Native sign-in requires Node.js 18 or newer and the opt-in `native_sign_in`
setting in the trusted tester `realm.json`.

The AWS rollout is pending. Deploy `TheiaDevelopmentNativeSignIn` first, then
update the existing Cognito stack with the helper's `NativePlayScopeArn`.
The trigger grants `theia-development/play` to `theia-testers` using the
designated launcher client. Use the [AWS runbook](../infrastructure/aws/README.md#native-launcher-authentication)
for rollout and rollback. Once deployed, enable `"native_sign_in": true` on a
tester config. Verify Play with a tester, a temporary password and MFA, denial
for a user outside `theia-testers`, and Play through the browser fallback.

## How the new release flow works

1. **Build Linux client release candidate** builds the locked Linux AMD64
   client and fully strips the distribution copy before measuring and hashing
   it. It packages only files listed in the tracked
   `release/linux-client-files.json`, checking each size and SHA-256. It emits
   the `theia-linux-client-release` Actions artifact.
2. **Publish Linux downloads to GitHub Pages** accepts a successful build
   from that workflow on `main`. It signs the Development feed, versions
   the installer, and also publishes the fixed URL
   `https://code-studio-au.github.io/theia-mmo/downloads/theia-launcher_amd64.deb`.
   In update mode it compares the live signed feed with the exact previous
   client version entered by the operator. It copies the previous versioned
   installer, referenced files, and optional binary patches into the new
   artifact. The retention window is one previous release. The current and
   previous assets must fit the Pages limit or the workflow fails before deploy.
3. The game publishing workflow verifies the live signed-feed bytes, stable
   installer, client executable, and signed news archive against the staged
   hashes after deploy.
   Launcher-only releases may reuse the exact client manifest and its patches.
   Client-only releases can reuse the existing versioned installer bytes.
   Supply the previous successful candidate as `delta_base_run` for binary
   patches against the published client. This is independent of retention.
4. The website builds without a Linux link by default. Once
   `ENABLE_LINUX_DOWNLOADS=true` is set in `theia-website`, its deploy workflow
   checks the live feed and stable installer before it replaces the website.
   Later game releases reuse the same link. Opening the launcher or checking
   updates never wakes the Sydney host.
5. Enter reviewed, plain-text notes for each product whose version changes.
   The game publisher adds one entry per changed version to the signed
   `release-news.json` archive. Earlier news survives even after its installers
   leave the retention window. The website imports the signed archive during
   its build and renders it in News and RSS. A matching hand-written article
   takes precedence. Once configured, the publisher triggers the website build
   only after live verification succeeds. If the trigger fails, retry the
   website workflow manually rather than republishing the game. There is no
   per-release update-manifest or website edit.

## Verification completed

- Python hosted-release suite passed over a local HTTPS server. This covered
  signed feeds, binary patching, corruption fallback, resume and rollback. The
  final retention scenario passed after the last changes: first publish,
  rejection when the expected prior version is wrong, second publish retaining
  the previous installer and patch, a byte-range download from the old release,
  and a launcher-only update.
- The candidate command runs directly from a fresh checkout. Tests export only
  the reviewed file bytes and reject untracked files or changed bytes. They use
  a Linux ELF fixture to verify section removal, manifest hashing and launch
  after installation. A separate local rehearsal exported the actual playtest
  executables with a test asset list. The 1,285,153,712-byte build became
  191,360,224 bytes. The signed local HTTPS update from 0.1.0.11 to 0.1.0.12
  downloaded only the 25,453,963-byte patch and verified the rebuilt hash.
- The Millhaven inventory produced a byte-reproducible 64,073,362-byte private
  build input with 3,719 assets. The local candidate builder verified that
  archive against a draft approval and installed the resulting 296,429,555-byte
  payload with the font notice. Its offline world configuration matched the
  playtest. The owner-only private build-input repository has a read-only
  deploy key. Clean Linux live admission remains pending.
- The read-only **Linux launcher release checks** workflow runs candidate,
  installer, launcher and hosted-release tests for relevant changes.
- Website `npm run check` passed. Static builds with the link off and on both
  passed, and the generated page was checked for the correct button or URL.
  The local environment used `ASTRO_TELEMETRY_DISABLED=1` as in CI.
- The signed release-news tests covered first publication, updates and a
  launcher-only release without an extra client post. The website importer
  rejected invalid signatures and duplicate entries, preserved the hand-written
  post, and rendered a generated post in the built index, article and RSS.
- PR #58 passed Rust and Linux launcher release CI. PR #59 passed
  [Rust](https://github.com/code-studio-au/theia-mmo/actions/runs/45572036507),
  [AWS infrastructure](https://github.com/code-studio-au/theia-mmo/actions/runs/45572036620),
  [Linux launcher release](https://github.com/code-studio-au/theia-mmo/actions/runs/45572036759)
  and [realm gateway](https://github.com/code-studio-au/theia-mmo/actions/runs/45572036564)
  checks before merging.
- The launcher `0.1.9` Debian package was extracted and exercised with GTK on
  Xvfb. Simulated Cognito responses covered inline errors, cancellation,
  temporary password changes, authenticator MFA, successful sign-in and
  browser handoff. Live Cognito tester verification remains pending.
- [Website PR CI](https://github.com/code-studio-au/theia-website/actions/runs/45557730120)
  passed on `dcd5583` using a PR build.
- YAML and embedded Bash syntax checks passed.

Useful focused checks from the respective checkout roots:

```sh
# theia-mmo
umask 022
python3 -m unittest discover -s tools -p test_hosted_release.py

# theia-website
npm ci
node --test scripts/test-import-release-news.mjs
ASTRO_TELEMETRY_DISABLED=1 npm run check
ASTRO_TELEMETRY_DISABLED=1 PUBLIC_LINUX_DOWNLOADS=false npm run build
ASTRO_TELEMETRY_DISABLED=1 PUBLIC_LINUX_DOWNLOADS=true npm run build
```

The game test needs Linux packaging tools, OpenSSL, zstd and loopback sockets.
Do not treat a local HTTPS fixture or a static website build as proof that
GitHub Pages is deployed.

## Resume here

1. Use [the main-only release environment](https://github.com/code-studio-au/theia-mmo/settings/environments)
   for candidate and publish runs. Keep the existing Ed25519 key stable. Use
   its public key for the website setup after the first verified publication.
2. Use the approved Millhaven input in `release/linux-client-files.json`.
   The list pins the archive and tracks additional notices by exact bytes.
   Dispatch the candidate with asset commit
   `2defc01ec05974cab79c7f22ddfceae16ee4ef1e`. This includes the approved player
   and wolf runtime files. The original source models remain outside the bundle.
   The Sydney test also used a separately supplied `realm.json`, CA certificate
   and launch arguments. Verify how the exact public package works on clean
   Linux, including any trusted realm setup needed by testers.
3. Review and merge website PR #1 with the website opt-ins unset. Build the
   game candidate on `main` and test the actual export on clean Linux. Record
   its successful run ID for generating the next release's binary patches.
   Dispatch the publisher in `first` mode only if the live feed is absent.
   Enter both launcher and client notes for a first release.
4. After live verification, set `ENABLE_LINUX_DOWNLOADS=true` and
   `ENABLE_RELEASE_NEWS=true` in `theia-website`. Set
   `THEIA_RELEASE_PUBLIC_KEY` there to the game release public key in base64
   Ed25519 DER format. Run the website workflow once and verify the download
   button, news and RSS. For subsequent automatic dispatch, set
   `ENABLE_WEBSITE_RELEASE_NEWS=true` and add `THEIA_WEBSITE_NEWS_TOKEN` in the
   **game repo's** `github-pages` environment. Use a fine-grained credential
   scoped to the website repo with Actions write permission, as described in
   [the release runbook](GAME_LAUNCHER.md#automatic-website-release-notes).
   Verify the launcher installs the signed client and the next release leaves
   a previous download resumable. Keep the signing key stable for updates.
5. Separately for automatic Sydney wake and idle stop, apply the reviewed
   identity stack with an AWS administrator, then follow
   [the gateway rollout runbook](../infrastructure/gateway/README.md). The
   existing host deploy role is limited to the host stack. Install the lifecycle
   controller and qualify two testers through stopped → wake → play → idle stop
   → wake, verifying saved progress. This rollout is independent of the Pages
   publishing mechanics.

Keep merges, public publishing and live AWS changes separate. Do not store or
print signing keys, AWS credentials or tester passwords in the repository.
