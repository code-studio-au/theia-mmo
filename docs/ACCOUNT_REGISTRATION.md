# Native Create Account — P3-01 / P2-14

23 November 2026. Builds on the native account bridge, owned Keycloak 26.8.1
provider, existing code/PKCE account service, PostgreSQL character persistence,
and approved v3 Bevy UI sprites. No browser, password grant, account administrator
credential or new gameplay protocol is introduced.

## Implemented behaviour

Login → Create Account opens a dedicated native screen using the supplied gateway
composition: illustrated adventurer, gold/navy panel and title, account name,
email, password and confirmation, independent eye controls, live password rules,
Create Account and one Back to Login button outside the panel at the bottom-left
of the screen. A fine gold divider with faded ends
separates the portrait and fields; the redundant existing-account login footer is
removed as requested. The portrait covers its clipped left bay to the inner left
and bottom frame edges, preserving its aspect ratio. The sword has been removed;
a separate white mist swirl behind the portrait softly fades into the navy
background at 65% opacity. Icon-bearing input text and
placeholders share a 52-unit left inset in both login and registration. The existing Theia Online title
and courtyard background retain the game's branding. The original illustration
is a visual reference, not a flattened interactive form. The native panel scales
to fit the actual client window; inputs share the standard 48-unit height.

Account names use 3–32 ASCII letters, digits, dots, underscores or hyphens.
Passwords use 8–128 characters, including an ASCII letter and digit, without
spaces/control characters. Password confirmation stays in the client. Both
password buffers clear after submission or leaving the page, and lose their
revealed state on focus loss. Credentials travel through the existing bounded,
capability-authenticated loopback socket, never the JSON menu-command file.
No email or password is emitted by UI evidence. This is bounded in-memory handling,
not a claim that every allocator/copy is securely zeroised.

The adapter obtains the provider's actual registration form using the same
code/PKCE authorization request, permits only its exact loopback origin/action,
and sends the provider redirect to the existing validated account callback.
The provider owns uniqueness and password/profile validation. On success, the
native client opens the new account's empty character roster. A failure retains
the form and a safe error; it never grants an account session optimistically.

The optional `Identity(..., registration=True)` fixture enables self-registration
and imports a declarative profile with only username/email, plus the password
policy. It does not fabricate first/last names. `edit_local.py` enables this for
new owned editor sessions and exports `THEIA_NATIVE_REGISTRATION=1`. Other providers
and existing sessions remain disabled unless explicitly configured; availability
is advertised in the public frontend snapshot. Reopen an existing local session
to use new provider configuration and client binaries.

Terms of Service, Privacy Policy and news subscription remain visibly disabled:
there is no supplied legal content, consent store or subscription service. They
are not falsely recorded as accepted. Recovery, verification email, MFA, deployed
identity, real subscriptions and launcher updates remain separate work. Owned
fixture accounts persist only while their provider data is retained; this is not
a deployed public registration service.

## Evidence

- 358 client tests passed; five existing skips. Includes native widget construction
  and registration name/email/password/confirmation guards.
- 12 Python native credential/bridge/editor-launch tests passed, including unknown
  fields, oversized inputs, capability refusal, provider refusal and hostile form
  action rejection before credentials are sent.
- `build/account-registration-provider-v3/qualification.json`: actual Keycloak
  registration and subsequent sign-in; duplicate, short/no-letter/no-digit/space
  password (including Unicode whitespace/control), malformed name and email
  refusals with adapter validation bypassed.
  This proves provider code issuance, not by itself a completed account exchange.
- `cargo fmt --all --check`, denied-warning release Clippy for all selected targets,
  release client/server builds and `git diff --check` pass.
- `build/account-registration-native-v3/qualification.json`: actual native form
  validation, registration/PKCE/account exchange, empty roster, character creation,
  PostgreSQL restart, duplicate-account refusal and ordinary sign-in restoring the
  same character. This does not qualify entry into gameplay.
- `build/account-registration-native-v6/qualification.json`: final return-button
  placement and click-back/reopen flow, fresh registration/character creation and
  1280×700 / 1720×700 / 1720×1000 visual/bounds checks. Final mist placement
  passes the same check in `build/account-registration-native-v7/qualification.json`;
  its empty, quarter and half-window screenshots were reviewed.
- Native runs use a private X display, Linux Vulkan / llvmpipe LLVM 21.1.8 /
  Mesa 26.2.7; they never drive the user's open game. They qualify real native
  rendering, not discrete-GPU performance or Windows execution. The full v3 fresh
  run counts one known `VUID-VkSwapchainCreateInfoKHR-pNext-07781` surface-resize
  diagnostic; resumed run counts zero. The final v7 resize check counts two of that
  exact diagnostic, within the existing explicit qualifier bound. No other GPU
  diagnostics are accepted.

## Source and artwork

Provider behaviour follows the installed 26.8.1 templates and
[Keycloak's server administration guide](https://www.keycloak.org/docs/26.8.1/server_admin/),
including self-registration and declarative user profiles. The two local fixture
qualifiers exercise those exact provider binaries.

`content/ui/frontend/account-art-generation.json` records generated portrait/mist/mail
artwork, prompt chain, deterministic export and SHA-256 hashes. Existing approved
v3 sprites supply the controls and ornament. Rejected contact-sheet/v1 ZIP assets
are not used. Private test accounts, screenshots and provider/database files stay
under ignored `build/` paths.
