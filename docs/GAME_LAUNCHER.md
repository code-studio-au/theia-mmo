# Theia Online launcher

The desktop launcher is a separate Linux AMD64 application with its own Debian
package and application menu entry. The package installs the launcher and its
artwork. The launcher then installs the game into a separate directory. The
native game window owns sign-in, registration and character selection.

## Local development use

Build the standalone launcher package with `dpkg-deb` available on the build
machine.

```sh
python3 tools/package_client_launcher.py build/theia-launcher_0.1.3_amd64.deb
```

Install the Debian package through the system's package installer and open
**Theia Online Launcher**. In **Settings**, choose a trusted client release
folder containing `client-release.json`, or a library containing versioned
subfolders. The default game installation is
`$XDG_DATA_HOME/theia-online/game` (normally `~/.local/share/theia-online/game`).
The launcher saves its folder choices in `launcher.json` beside that default
directory.

**Install Game** stages and verifies the selected release, then enables **Play**.
Play opens the game fullscreen by default and minimises the launcher. When the
game closes, the launcher is available again. Forward `--window-size WIDTHxHEIGHT`
after `--` for a windowed launch or use F11 to toggle fullscreen in the client.
When a newer release is found, the main action becomes **Update Game**.
**Updates** also offers **Check for Updates**, **Restore Previous Version** and
**Play Installed Version** while a newer release is offered. Install and update
show verified bytes, percentage and, after enough time to measure the transfer,
a rate and estimated time left. Cancel stops the transfer. The bottom bar
displays the active client version.

The Play screen links to the bundled 0.1.0.1 client preview announcement.

## Launcher updates

Install the 0.1.3 launcher package once to enable checks for future launcher
releases. At startup and on **Updates → Check for Updates**, the launcher looks
in the `launcher/` subfolder of the selected trusted release library for Debian
packages named `theia-launcher_VERSION_amd64.deb`. It checks the package name,
version and architecture, then offers the newest compatible version above the
running version. The Updates tab displays launcher and game client versions
independently, and Play shows a link to any available launcher update.

Select **Update Launcher** to install the offered package with the system APT
installer and desktop administrator approval. **Restart Launcher** becomes
available after the system confirms the offered version is installed. The app
keeps its settings, game installation and launch arguments across the restart.
Publish packages only to a library you trust. This path uses local files.

## Login server status

The launcher probes the development account service's `GET /ready` every five
seconds. Green **Online** requires the account service's readiness response.
Other responses or connection failures show **Unavailable**. The default is
`127.0.0.1:5050`. With a custom numeric loopback bind, set
`THEIA_ACCOUNT_BIND` for the launcher or pass `--login-server 127.0.0.1:PORT`.
Use the matching current account server build.

The endpoint is available on the existing guarded account listener once
startup has completed. The probe sends no credentials, starts no login session
and does not test a complete identity-provider sign-in. Public realm
monitoring remains separate.

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
The launcher estimates only missing bytes and downloads files by hash. It reuses
verified installed files even if their paths moved. Partial downloads can resume
when the server supports byte ranges. A failed signature, failed hash check,
cancelled download or incomplete package leaves the active version in place.
Launcher 0.1.8 can also download a binary patch for a changed large file.
The publishing workflow accepts an optional previous successful Linux client
export via `delta_base_run` (or `--delta-base` when staging locally). It stages
Zstandard patches only when they save at least 20% and fit the Pages size
budget. Patch metadata is signed separately so the original release format
remains readable. The launcher checks the installed file's hash, verifies the
downloaded patch, and verifies the complete rebuilt file against the release
manifest before activation. A missing patch, mismatched base, invalid signature
or corrupt patch falls back to the full changed file. A partial patch can
resume on servers that support byte ranges. Full files remain available for
clean installs. The launcher uses `zstd` when installed.

Local folders are still selectable in Settings.

The separate Pages project keeps packages available across website builds.
The public website links to a stable `downloads/theia-launcher_amd64.deb` URL.
Each publish points that file at the current installer and preserves its
versioned copy, plus the previous installer, client files and patch files.
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
The executable-only private test export is insufficient as a public payload.

1. Enable GitHub Actions as the Pages source on `theia-mmo`. Protect the
   `game-downloads-release` environment and configure its Ed25519
   `THEIA_RELEASE_SIGNING_KEY` secret.
2. Dispatch **Build Linux client release candidate** on `main`. It verifies
   the approved files by size and hash and uploads `theia-linux-client-release`.
   Smoke-test the exact export on a clean Linux install before approving it.
3. Dispatch **Publish Linux downloads to GitHub Pages** with the candidate run
   and versions. Choose `first` only when the feed does not exist. For an
   `update`, enter the exact previous client version. The publisher verifies
   it and carries its live files forward before replacing the Pages artifact.
   The optional `delta_base_run` generates patches and is independent of retention.
4. After post-deploy verification passes for the live feed, stable installer
   and executable, set `ENABLE_LINUX_DOWNLOADS=true` in `theia-website` and
   dispatch the website deployment. The site build verifies the live links.
   Future releases use the same website link.

The previous-release retention window covers one rollout. Installed releases
can still roll back locally. Client-only releases can reuse the existing
installer. Launcher-only releases can reuse the exact client manifest and its
published patch files. Changes to either artifact require advancing its version.
Local HTTPS tests cover install, partial transfer resume, executable-only and
single-zone updates, corruption rejection and rollback. The update tests assert
that no unrelated UI, settings or terrain bytes are requested. A public download candidate and
hosted realm configuration remain to be qualified. Windows needs its own
installer and native qualification.

## Test sign-in without a game server

P3-01 adds an opt-in launcher sign-in rehearsal. The Account tab opens the
system browser and completes the authorization-code exchange with PKCE through
a loopback callback. The provider confirms the account using the access token.
The launcher reports success without retaining the token, waking a realm or
launching a game session.

Run the two-account check against the existing owned Keycloak SDK:

```sh
python3 tools/qualify_launcher_test_sign_in.py
```

Add `--desktop` to open the launcher and select **Account → Test Sign-in**.
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
