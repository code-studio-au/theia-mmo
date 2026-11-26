# Theia Online launcher

The desktop launcher is a separate Linux AMD64 application with its own Debian
package and application menu entry. The package installs the launcher and its
artwork. The launcher then installs the game into a separate directory.
**Play** opens hosted tester sign-in when a realm configuration is supplied.
The game owns character selection and any enabled account registration.

## Local development use

Build the standalone launcher package with `dpkg-deb` available on the build
machine.

```sh
python3 tools/package_client_launcher.py build/theia-launcher_amd64.deb
```

Install the Debian package through the system's package installer. To use
a trusted local release or a directory of versioned exports, start the
launcher with `theia-launcher --release-library /path/to/releases`. The default
game installation is
`$XDG_DATA_HOME/theia-online/game` (normally `~/.local/share/theia-online/game`).
The launcher saves the game folder in `launcher.json` beside that default
directory. Standard launches use GitHub Pages for online releases and sign in
to Theia with no command-line options. Use `--realm-config` to test another
realm. Use `--local-sandbox` to play an offline sandbox with a development
client build. Selecting a local release library changes the install source.

**Install Game** stages and verifies the selected release, then enables **Play**.
Play opens the game fullscreen by default and minimises the launcher. When the
game closes, the launcher is available again. Forward `--window-size WIDTHxHEIGHT`
after `--` for a windowed launch or use F11 to toggle fullscreen in the client.
When a newer release is found, the main action becomes **Update Game**.
**Settings** has **Check for Updates**, the game folder, and the launcher
version. To expose **Restore Previous Version** and **Play Installed Version**
for development, start with `--developer-tools`. Install and update show
progress and an estimated time left. Cancel stops the transfer. The bottom bar
displays the active client version.

**News** lists recent articles from the public site's RSS feed. **Play** shows
the latest article, and selecting an article opens it in the browser.

## Launcher updates

Launcher 0.1.11 checks the signed GitHub Pages `launcher.json` feed at startup
and on **Settings → Check for Updates**. A newer version takes priority over
**Play**, **Install Game**, and **Update Game**. The launcher downloads the
versioned Debian package and verifies its signed size and SHA-256, then checks
the Debian package name, version, and architecture. Versions before 0.1.11
require the website's installer once to adopt this update path.

Select **Update Launcher** to install the offered package with the system APT
installer and desktop administrator approval. **Restart Launcher** becomes
available after the system confirms the offered version is installed. The app
keeps its settings, game installation and launch arguments across the restart.
The explicit `--release-library` mode discovers trusted packages from that
directory's `launcher/` subfolder for offline development.

## Realm status

Use **Theia** as the realm and server name in the launcher, deployment
announcements, and release news. **Millhaven** is the test zone on Theia.
Sydney identifies the hosting location.

**Realm Status** appears during normal public startup and with a hosted realm
override. The launcher verifies the signed `downloads/realm.json` profile and
requires its client version for Play. If the profile is unavailable or fails
verification, Play stays disabled and **Check for Updates** can retry. The launcher probes
the realm's account service every five seconds and shows **Online** when the
service is ready. If the realm supports wake on demand and the service is
unreachable, the launcher shows **Sign in to start**. Play can request startup
through the gateway after successful sign-in.

Local development can display **Login Server Status** via `--login-server`,
`THEIA_ACCOUNT_BIND`, the local library, or developer tools. The default
loopback address is `127.0.0.1:5050`. Readiness probes send no credentials and
never wake a realm.

## Preparing a client release

Export each validated build to a new version folder in the same library.

```sh
python3 tools/client_install.py export build/launcher-staging build/client-releases/0.1.0.12 --version 0.1.0.12
theia-launcher --release-library build/client-releases -- --window-size 1280x720
```

Stage the compiled `theia-client` and the assets required by that build. In
development, use the normal private account playtest to start and configure
matching local services. Pass its usual client arguments after `--` when
starting the launcher. The same local installer also exposes the `install`,
`status`, `rollback` and `play` subcommands for command-line use.

## Activation and testing

The installer verifies the platform, SHA-256 and size of each file before
switching versions atomically. Installation failures and cancellation preserve
the active version. A desktop update cannot activate an older release after a
concurrent update or silently install a different version from the one offered.
Launch verifies the installed files and starts the game from its active version
directory. A running game can continue using its version files.

Sixteen installer/launcher checks and 25 server library tests pass. The
packaged app was exercised on Linux X11 with two distinct local client builds,
a rejected corrupt update and rollback. Captures cover game not installed,
installing, update required, updating and game ready, with real byte progress.
The 967×600 layout and top-right maximise, restore and minimise controls were
checked. The launcher reported Online against the running private account
service. The installed client then opened native Login at 1280×720 and signed
in to a fresh account roster. Linux Vulkan llvmpipe reported zero validation
diagnostics. See `build/launcher-theme-qualified-65dre1f2/qualification.json`.

The 0.1.2 package opened installed client 0.1.0.4 fullscreen at 1440×900 on
Linux X11 with Vulkan llvmpipe. The launcher minimised on Play and restored
when the client closed. All 19 installer and launcher checks passed.

For 0.1.3, all 22 checks passed with an APT upgrade simulation. On private
Linux X11, the packaged GTK app detected a newer launcher on startup and on
manual check, installed it into an isolated Debian package root and restarted
into the updated app with game 0.1.0.4 still active. The test substituted the
isolated Debian install for the live desktop administrator prompt.

## Development downloads on GitHub Pages

P6-09 adds an online release source for Linux AMD64. A launcher package built
with an Ed25519 public key checks the Development feed on the game repository's
GitHub Pages project. It verifies the signed metadata before offering **Install
Game** or **Update Game**. The feed versions the executable, UI, character,
equipment, animation, objects, audio, gameplay and each terrain zone independently
using hashes of their file entries. Organize an export under `ui/`, `characters/`,
`equipment/`, `animations/`, `objects/`, `audio/` and `terrain/<zone>/`, optionally
under `assets/` or `content/`. Other files belong to the gameplay package.
Launcher 0.1.10 uses the signed `bundles/<client-version>.json` index for a
fresh install. It downloads a ZIP for the executable and a ZIP for the assets,
checks the archive hashes, and verifies every extracted file against the signed
client manifest before activation. Releases without a usable bundle index or
with a corrupt bundle can still install from individual files.

For updates, the launcher estimates only missing bytes and downloads files by
hash. It reuses verified installed files even if their paths moved. The
launcher automatically retries interrupted file transfers twice. When the server
supports byte ranges, each retry continues from the saved offset. If the retries
run out, choose **Install Game** or **Update Game** again in the same game folder.
A failed signature, failed hash check, cancelled download or incomplete package
leaves the active version in place.
Launcher 0.1.8 can also download a binary patch for a changed large file.
The publishing workflow accepts an optional previous successful Linux client
export via `delta_base_run` (or `--delta-base` when staging locally). It stages
Zstandard patches only when they save at least 20% and fit the Pages size
budget. Patch metadata is signed separately so the original release format
remains readable. The launcher checks the installed file's hash, verifies the
downloaded patch, and verifies the complete rebuilt file against the release
manifest before activation. A missing patch, mismatched base, invalid signature
or corrupt patch falls back to the full changed file. A partial patch can
resume on servers that support byte ranges. The launcher uses `zstd` for
binary patches when installed.

The separate Pages project keeps packages available across website builds.
The public website links to a stable `downloads/theia-launcher_amd64.deb` URL.
Each publish points that file at the current installer and preserves its
versioned copy, plus the previous installer, client files, install bundles and
patch files. The publish workflow checks the live signed bundle index and the
hash of each archive before it triggers the website rebuild.
The update feed stays at `downloads/development.json`. Keep using the
existing signing key for updates; the current publisher requires it to verify
the live feed. If both releases do not fit the Pages budget, publishing fails
before changing the live site.
Prepare and inspect an artifact locally with a disposable testing key:

```sh
python3 tools/stage_hosted_release.py path/to/client-export build/game-pages-preview \
  --launcher-version 0.1.5 --signing-key /private/path/test-ed25519.pem
```

This produces a Pages directory with the `.deb`, its checksum, a signed
`downloads/development.json` and the referenced client files. Use a new directory
for each rehearsal. Keep signing keys outside the repo and use the production key
only in the protected publishing environment. The command also supports
`--retain-client` for previously reviewed client exports whose files must stay
available. Use new launcher and client versions for changed packages.

For publication, review the files needed to run the Linux client on a clean
Ubuntu 24.04 or Linux Mint 22 system. Add only redistribution-approved files to
`release/linux-client-files.json` on `main`. It contains a `files` array of
`source`, `path`, `bytes` and `sha256` entries. Sources must be tracked under
`content/`, `apps/client/assets/` or `release/`. Paths are relative to the
installed payload. The locked Linux client build supplies `theia-client`.
The candidate exporter uses GNU binutils to fully strip the distribution copy
before measuring or hashing it. The Cargo output keeps its debug information.
Treat the unstripped build or separate debug symbols as private diagnostics.
The executable-only private test export is insufficient as a public payload.

Generated runtime assets can come from one reviewed build input. Freeze the
inventory's selected payload paths from an existing Linux export with:

```sh
python3 tools/linux_client_asset_bundle.py /path/to/client-export paths.txt theia-linux-assets.tar.gz
```

`paths.txt` contains one reviewed payload path per line. The command verifies
those bytes against the original export manifest and prints the archive byte
count and SHA-256. Add that object as `bundle` in
`release/linux-client-files.json` alongside the `files` array. Keep required
notices, including `content/ui/fonts/LICENSE.txt` for Liberation fonts, among
the reviewed files. Put the archive at the root of the configured private
GitHub build-input repository. Configure `THEIA_LINUX_ASSETS_REPOSITORY` and a
`THEIA_LINUX_ASSETS_SSH_KEY` secret in the `game-downloads-release` environment.
Register the matching public key as a read-only deploy key on the asset
repository. Pass the exact Git commit as
`asset_commit` when building. The workflow verifies the pinned archive and
exports separate files. The launcher still downloads individual content blobs.

Deliver client builds for playtesting through the signed GitHub Pages feed so
testers can install and update from the launcher. A client release is ready
when the public update is verified and the hosted realm accepts its version.

Theia's public playtest uses the full authored gameplay export. Generate the
client and server arguments with `python3 tools/theia_playtest_profile.py
<client|server> <payload>`. Launch uses the signed profile automatically.
The release check verifies NPC contacts, loot, inventory, journal, progression,
combat abilities and effects, chat, parties, and the terrain-matched Covenant
map, which covers seven zones including Millhaven. Keep the approved map and
spell feedback data in `release/theia/`.

1. Enable GitHub Actions as the Pages source on `theia-mmo`. Protect the
   `game-downloads-release` environment and configure its Ed25519
   `THEIA_RELEASE_SIGNING_KEY` secret.
2. Dispatch **Build Linux client release candidate** on `main`. It verifies
   the approved files and optional asset bundle by size and hash and uploads
   `theia-linux-client-release`.
   Smoke-test the exact export on a clean Linux install before approving it.
3. Dispatch **Publish Linux downloads to GitHub Pages** with the candidate run
   and versions. Choose `first` only when the feed does not exist. For an
   `update`, enter the exact previous client version. The publisher verifies
   it and carries its live files forward before replacing the Pages artifact.
   Set `THEIA_PUBLIC_REALM_PROFILE` on the `game-downloads-release` environment
   to the reviewed public Theia profile. Include the deployed client's version,
   its runtime arguments, the public Cognito and gateway details, and the
   public TLS certificate in PEM form as `certificate`. The signed profile is
   published with the client. Keep tester credentials and private keys outside
   this profile. Staging requires a version match and checks the referenced
   runtime paths against the approved payload.
   For a client update, supply the previous successful candidate as
   `delta_base_run` to generate patches against the exact bytes already installed.
   The executable must fit the 256 MiB patch limit in both exports. Verify the
   offered download and rebuilt hash before relying on a patch in rollout.
4. After post-deploy verification passes for the live feed, stable installer
   and executable, set `ENABLE_LINUX_DOWNLOADS=true` in `theia-website` and
   dispatch the website deployment. The site build verifies the live links.
   Future releases use the same website link.

The candidate workflow builds the client with `public-client`. Bare starts and
development preview modes require a development build. The public build allows
the authenticated launcher entry and the headless configuration report.

The previous-release retention window covers one rollout. Installed releases
can still roll back locally. Client-only releases can reuse the existing
installer. Launcher-only releases can reuse the exact client manifest and its
published patch files. Changes to either artifact require advancing its version.
## Automatic website release notes

When publishing Linux downloads, enter reviewed public notes for whichever of
`launcher_version` or `client_version` changed. Use one plain-text change per
line. Leave notes blank for an unchanged product; on first publish, supply
both. The publisher signs the download feed, any patch metadata, and a
separate `release-news.json` archive. The archive preserves prior news even
when packages fall out of the previous-release retention window. Updates
fail before deployment if signed news history is missing or does not match
the live release. Review every line for public disclosure.

One-time setup after the first verified publish:

1. In `theia-website`, set `ENABLE_RELEASE_NEWS=true` and set the
   `THEIA_RELEASE_PUBLIC_KEY` repository variable to the existing game release
   signing **public** key as base64-encoded Ed25519 DER (SPKI). Dispatch
   the normal website workflow once to backfill the posts. The website uses
   the public key only.
2. To dispatch future website builds automatically, set
   `ENABLE_WEBSITE_RELEASE_NEWS=true` on the game publisher's `github-pages`
   environment. Add the `THEIA_WEBSITE_NEWS_TOKEN` secret there. Use a
   fine-grained token authorized for `code-studio-au/theia-website` with
   **Actions: write** only (plus required Metadata read). This token runs
   the existing website workflow on main after the game files and news are
   verified live. It does not need Contents write or access to AWS.

If the automatic dispatch fails, the game release is already live. Retry
the website workflow manually; it reimports the same archive and produces the
same articles and RSS entries. Do not republish the same game versions. A
hand-written article with the matching version takes precedence in the
website build. No per-release edit to the website or the download manifest
is required.


Local HTTPS tests cover install, partial transfer resume, executable-only and
single-zone updates, corruption rejection and rollback. The update tests assert
that no unrelated UI, settings or terrain bytes are requested. A public download candidate and
hosted realm configuration remain to be qualified. Windows needs its own
installer and native qualification.

## Test sign-in without a game server

P3-01 adds an opt-in launcher sign-in rehearsal. The Settings test control opens the
system browser and completes the authorization-code exchange with PKCE through
a loopback callback. The provider confirms the account using the access token.
The launcher reports success without retaining the token, waking a realm or
launching a game session.

Run the two-account check against the existing owned Keycloak SDK:

```sh
python3 tools/qualify_launcher_test_sign_in.py
```

Add `--desktop` to open the launcher and select **Settings → Test Sign-in**.
The command prints the path of a private file with two temporary test accounts.
The provider and credentials are removed when the rehearsal closes.

A future Cognito test app can be used with `--test-login-config path/to/sign-in.json`.
The JSON fields are `issuer`, `authorization_endpoint`, `token_endpoint`,
`userinfo_endpoint`, `client_id` and `redirect_uri`. Register the exact
loopback callback and enable the authorization-code grant with `openid` and
PKCE for a public app client. Use HTTPS for provider endpoints outside the
owned loopback fixture. Hosted realm admission and live Cognito qualification
remain separate from the local rehearsal.

## Sydney development realm

Launcher 0.1.6 accepts `--realm-config`. **Play** checks for client 0.1.0.12,
opens Cognito managed login in the browser, and exchanges the scoped access
token for an in-memory realm session. The native client opens at the account's
character roster. Character entry uses a short-lived Renet admission ticket.

On the development machine, open the prepared launcher with:

```sh
./build/hosted-development-launch/open-launcher
```

The client export and launcher Debian package are in `build/launcher-client-exports/`.
A tester using the installed launcher needs the client export, `realm.json`, and
the adjacent `realm-ca.crt`. Start the launcher with `--release-library` pointing
to the client export and `--realm-config` pointing to the JSON file, then install
the client. Provision accounts through Cognito and distribute credentials privately.
The launcher uses the bundled CA certificate for the realm's HTTPS endpoint.
The only public game ports are TCP 443 and UDP 5000. PostgreSQL and the account
backend are bound to loopback. The [gateway rollout runbook](../infrastructure/gateway/README.md)
covers opt-in automatic wake and idle stop. Live gateway rollout and the two-tester
wake, play, idle stop and progress restoration check remain pending.

### Native Cognito sign-in

**Play** opens a themed sign-in panel beside the artwork and latest news. It
uses Cognito SRP and supports temporary password changes and SMS or
authenticator app codes. **Use browser** opens managed login for account recovery and
MFA enrollment. Passwords and Cognito tokens stay in process memory. The
gateway and game server still validate the tester group and play scope.

The Pages workflow builds and includes the native helper in the signed launcher
package. For local source testing, run the following from the repository root.
The installed launcher uses `/usr/bin/node` version 18 or later. Browser sign-in
opens directly if the native helper or Node.js is missing.

```sh
npm --prefix apps/launcher/native-auth ci
npm --prefix apps/launcher/native-auth run build
npm --prefix apps/launcher/native-auth test
```

After [deploying native authentication in Sydney](../infrastructure/aws/README.md#native-launcher-authentication),
set the top-level field `"native_sign_in": true` in the trusted `realm.json`
passed through `--realm-config`. Verify native Play with a tester account, a
temporary password, and configured MFA. Verify that a user outside
`theia-testers` is denied and that a tester can still play through managed
browser login. Remove the field to return that launcher to browser login.

## Launcher artwork

The launcher uses a graded [harbour](../content/ui/frontend/launcher-harbour-v2.png)
and [adventurer portrait](../content/ui/frontend/launcher-adventurer-v2.png).
Both were edited with the built-in ImageGen tool using the website's homepage
art as the lighting and colour reference.

**Harbour prompt**

```text
Use case: style-transfer.
Asset type: wide fantasy harbour background for the Theia launcher.
Image 1 is the exact edit target. Image 2 is a lighting and color reference
from the website only.
Apply an extremely subtle grade to image 1. Soften the brightest sky, pale
stone and lake reflections, pull vivid electric blue and cyan back slightly
toward the website's quieter blue and slate, and give the light the
reference's warm natural daylight. Keep the image luminous and readable with
open shadows and crisp details. Preserve the original very wide 2.1:1
framing, left tree and arch, bridges, mountains, city, banners, lake and
crop as precisely as possible. Preserve the whole composition and all
existing content. Add no people, words, overlays, or new objects. Output
only the edited opaque landscape, as close to the original dimensions as
possible.
```

**Adventurer prompt**

```text
Use case: identity-preserve.
Asset type: existing foreground character overlay for the Theia launcher.
Image 1 is the exact edit target, the brunette woman in navy-and-gold
Covenant armour. Image 2 is the website's reference for lighting and colour
only.
Give image 1 the same extremely subtle softer grade as the website: pull
back shiny gold highlights a little, slightly quiet the bright blue, and
keep the face naturally warm and clearly legible. This is a colour and
contrast pass, not a redesign. Keep the exact woman, face, hair, expression,
proportions, profile pose looking right, blue cape and tree emblems,
clothing and silhouette. Keep her in the same place and the same tall 2:3
crop, with the same soft edge treatment and transparent surround fit for
layering over the harbour. Output a genuinely transparent PNG. Do not add a
scene, extra characters, or text.
```
