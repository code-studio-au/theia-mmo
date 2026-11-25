# Launcher download handoff, 25 November 2026

## Resume here

Use the bundled GitHub Pages download as the default fresh install for P6-09.
The user stopped the old per-file download and will restart with the bundled
launcher to test the fresh-install experience and then native Sydney sign-in.
The user will install and open the launcher and press Play themselves.

The public download still serves launcher 0.1.9 and client 0.1.0.13. The
stopped download's `/home/steven/Games/.hosted-download` was deleted at the
user's request, freeing about 242 MB. Use a separate empty install root for
the new fresh-install test. The verified 0.1.0.13 export is
in `build/launcher-client-exports/0.1.0.13`. Its successful GitHub candidate run
is [45593504068](https://github.com/code-studio-au/theia-mmo/actions/runs/45593504068).

## Local checkpoint, 26 November 2026

`git status` on `fix/sydney-gateway-stage-tags` shows the local launcher,
release work, tests and documentation. Launcher 0.1.10 uses the signed bundle
index for a fresh install. It verifies the two ZIPs and every extracted file
before activation. Installed clients reuse unchanged files and fetch changed
files or binary patches. Transient file transfer errors are retried twice from
the saved offset. The Pages workflow now stages bundles and checks their live
hashes alongside the feed and launcher before requesting a website rebuild.

The 37 launcher and release tests and four native sign-in helper tests passed.
The release tests cover interruptions, corrupt-bundle fallback, changed-file
updates, and retention across a client release and a launcher-only release.
The exact 0.1.0.13 export matched the public signed feed. A loopback HTTPS
rehearsal using a temporary signing key resumed an interrupted ZIP at 256 KiB
and verified all 3,721 staged files. It fetched both ZIPs and made no
individual file requests. The executable ZIP is 93,667,970 bytes, the assets
ZIP is 65,036,007 bytes, and the rehearsed Pages artifact with the retained
0.1.9 installer is 493,310,404 bytes. `cargo fmt --check` passed. Workspace
Clippy could not start its ALSA and Wayland builds because the development
libraries are missing.

The local review package is
`build/launcher-client-exports/launcher-0.1.10/theia-launcher_0.1.10_amd64.deb`.
Its launcher source, native sign-in helper, and production public key were
verified after package extraction. The public site still serves 0.1.9 and
the 0.1.0.13 per-file release.

## Next checks and release

1. After push and publication are authorized, merge the launcher changes
   through the Linux launcher release checks. Publish launcher 0.1.10 in
   update mode, using candidate run `45593504068`, with both the client version
   and expected prior version set to `0.1.0.13`. Supply reviewed launcher
   notes and leave client notes blank because the client version is reused.
2. Verify the Pages checks and stable download. The user can then install into
   a fresh folder and test **Play → Sign in** using the existing private realm
   config at `build/hosted-candidate-launch-0.1.0.13/realm.json`.

The last confirmed Sydney lifecycle check showed a successful automatic idle
stop. The cold wake, character roster and saved progress are part of the
user's playtest. The earlier [Linux release handoff](HANDOFF_LINUX_RELEASE_2026-11-25.md)
still covers the release workflows and site setup.
