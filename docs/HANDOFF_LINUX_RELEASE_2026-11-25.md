# Linux launcher and Pages handoff · 25 November 2026

The stable public download link and safe Pages release flow are implemented and
locally qualified. The next step is to review the two branches and prepare the
first approved Linux client payload. The public release and the AWS wake rollout
have not been activated.

## Checkouts

| Repo | Branch to fetch | Contents |
| --- | --- | --- |
| [code-studio-au/theia-mmo](https://github.com/code-studio-au/theia-mmo/tree/development/safe-pages-releases) | `development/safe-pages-releases` | Release candidate build, safe Pages publishing and this handoff. Implementation starts at `1b4c5d57`, based on `main` at `a0ae08d4`. |
| [code-studio-au/theia-website](https://github.com/code-studio-au/theia-website/tree/development/verified-linux-download-link) | `development/verified-linux-download-link` | Website opt-in, pre-deploy checks and stable Linux link. Commit `8886852`, based on `main` at `25c61f6d`. |

Review the [game release draft](https://github.com/code-studio-au/theia-mmo/pull/58)
and [website draft](https://github.com/code-studio-au/theia-website/pull/1).
Check the latest remote state and preserve any local work before switching branches. Neither site needs to be deployed to
review these changes. Read [the game launcher runbook](GAME_LAUNCHER.md) and
[the website deployment notes](https://github.com/code-studio-au/theia-website/blob/development/verified-linux-download-link/README.md).

## What is live

- `https://theiaonline.com/download/` is the website project on GitHub Pages.
  The Linux download is currently unavailable there. The live page was checked
  and had no Debian package link.
- The launcher expects
  `https://code-studio-au.github.io/theia-mmo/downloads/development.json`.
  This separate project returned HTTP 404 for the feed. The game download
  publishing workflow had not run, GitHub did not report a game Pages site,
  and there was no `theia-linux-client-release` artifact when checked.
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

## How the new release flow works

1. **Build Linux client release candidate** builds the locked Linux AMD64
   client. It packages only files listed in the tracked
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
   installer, and client executable against the staged hashes after deploy.
   Launcher-only releases may reuse the exact client manifest and its patches.
   Client-only releases can reuse the existing versioned installer bytes.
   `delta_base_run` is optional and independent of retention.
4. The website builds without a Linux link by default. Once
   `ENABLE_LINUX_DOWNLOADS=true` is set in `theia-website`, its deploy workflow
   checks the live feed and stable installer before it replaces the website.
   Later game releases reuse the same link. Opening the launcher or checking
   updates never wakes the Sydney host.

## Verification completed

- Python hosted-release suite passed over a local HTTPS server. This covered
  signed feeds, binary patching, corruption fallback, resume and rollback. The
  final retention scenario passed after the last changes: first publish,
  rejection when the expected prior version is wrong, second publish retaining
  the previous installer and patch, a byte-range download from the old release,
  and a launcher-only update.
- The candidate command runs directly from a fresh checkout. Tests export only
  the reviewed file bytes and reject untracked files or changed bytes. They use
  a Linux ELF fixture rather than the public game build.
- The read-only **Linux launcher release checks** workflow runs candidate,
  installer, launcher and hosted-release tests for relevant changes.
- Website `npm run check` passed. Static builds with the link off and on both
  passed, and the generated page was checked for the correct button or URL.
  The local environment used `ASTRO_TELEMETRY_DISABLED=1` as in CI.
- YAML and embedded Bash syntax passed for the three changed workflows.

Useful focused checks from the respective checkout roots:

```sh
# theia-mmo
umask 022
python3 -m unittest discover -s tools -p test_hosted_release.py

# theia-website
npm ci
ASTRO_TELEMETRY_DISABLED=1 npm run check
ASTRO_TELEMETRY_DISABLED=1 PUBLIC_LINUX_DOWNLOADS=false npm run build
ASTRO_TELEMETRY_DISABLED=1 PUBLIC_LINUX_DOWNLOADS=true npm run build
```

The game test needs Linux packaging tools, OpenSSL, zstd and loopback sockets.
Do not treat a local HTTPS fixture or a static website build as proof that
GitHub Pages is deployed.

## Resume here

1. Review and merge the two review branches. The website stays in its current
   public state with the opt-in variable unset.
2. Assemble a redistribution-approved, self-contained Linux payload. Commit
   `release/linux-client-files.json` with each file's tracked source path,
   destination relative to the installed payload, byte count and SHA-256. The
   asset list is deliberately absent. Do not publish the executable-only
   private test export, locally licensed assets without approval, private
   account configuration or credentials. Review assets embedded by the client
   build as well. The current Sydney test uses a separately supplied
   `realm.json`, CA certificate and launch arguments. Qualify the exact public
   package from a clean Linux install, including how testers pick up trusted
   realm configuration if that is in scope.
3. Enable Pages using GitHub Actions on `theia-mmo`. Configure the protected
   `game-downloads-release` environment and its Ed25519
   `THEIA_RELEASE_SIGNING_KEY` secret. Build a candidate on `main` and verify
   the exact artifact before dispatching the manual publisher in `first`
   mode. Do not use that mode if a feed has appeared since this handoff.
4. After live verification passes, set `ENABLE_LINUX_DOWNLOADS=true` in
   `theia-website` and deploy it. Verify the public button downloads the package,
   the launcher installs the signed client on clean Linux, and a later release
   leaves a previous download resumable. Keep the signing key stable for updates.
5. Separately for automatic Sydney wake and idle stop, apply the reviewed
   identity stack with an AWS administrator, then follow
   [the gateway rollout runbook](../infrastructure/gateway/README.md). The
   existing host deploy role is limited to the host stack. Install the lifecycle
   controller and qualify two testers through stopped → wake → play → idle stop
   → wake, verifying saved progress. This rollout is independent of the Pages
   publishing mechanics.

Keep merges, public publishing and live AWS changes separate. Do not store or
print signing keys, AWS credentials or tester passwords in the repository.
