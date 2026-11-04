# Release builds

`.github/workflows/chiacraft-build.yml` builds chiacraft for Windows, Linux and macOS on GitHub's
runners and packs each into one archive a player unpacks and runs. It is ours, beside upstream
Weft's `release.yml` (which builds plain Weft on `v*` tags), so syncing with upstream never
touches it.

## Starting a build

- **By hand:** GitHub > Actions > "chiacraft build" > Run workflow. Pick the branch to run from,
  and optionally:
  - `ref`: another branch, tag or commit to build (it has to contain `tools/release/`);
  - `platforms`: any of `windows-x64 linux-x64 macos-arm64 macos-x64`, space-separated. The
    default leaves out Intel Macs.

  From a terminal: `gh workflow run chiacraft-build.yml --ref dev` (add `-f platforms=linux-x64`
  for one). The archives are the run's artifacts, kept 90 days; download them from the run's
  page or with `gh run download <run id>`.
- **On a tag** `chiacraft-v<version>` (e.g. `chiacraft-v0.1.0`): all four platforms, then a
  GitHub **pre-release** named after the tag with every archive attached. Nothing is published
  unless every build passed. Plain `v*` tags start upstream's `release.yml` instead, which fails
  on our version numbers.

A manual run shows up in the Actions tab only once the workflow file is on the default branch
(`main`). From then on it can run from any branch that has it.

**Cost.** The repository is private, so runs use the account's Actions minutes, and Mac minutes
count 10 times, Windows twice. A full release build takes about 20-40 minutes per platform, so
one run with both Macs can use several hundred of the included minutes. Build only the platforms
you need (`-f platforms=...`); the cargo cache makes later runs of the same branch faster.

## What comes out

`chiacraft-<label>-<platform>.tar.gz` (`.zip` on Windows), where `<label>` is the tag's version,
or `<branch>-<commit>`. Inside, one folder `chiacraft/`:

| file | what |
|---|---|
| `weft` / `weft.exe` | the game |
| `assets/core/` | the engine's assets (shaders, textures, fonts, sounds, languages) |
| `run/mods/*.zip` | our mods, each with its library and assets: base (which carries the chiacraft shader pack and writes it to `shaderpacks/chiacraft/` on the first start), animals, portals, packages. The test kit and testmod are left out (`dist = false`). |
| `libMoltenVK.dylib`, `MoltenVK-LICENSE` | macOS only: Vulkan on Metal, so players need no Vulkan SDK |
| `README.txt`, `BUILD.txt`, `LICENSE` | how to run it, which commit (and on macOS, whether it is signed) |

The game itself makes this folder (`weft --dist`); `tools/release/package.sh` runs that, checks
the result and archives it. The same script runs anywhere the game builds:

```
cargo build --release
tools/release/package.sh linux-x64      # dist/chiacraft-<commit>-linux-x64.tar.gz
```

What each runner needs to build: Linux `libasound2-dev pkg-config` (ALSA for sound); Windows
NASM (for the wallet's TLS library, aws-lc-sys) and MSVC, which the runner has; macOS nothing
beyond Xcode's tools, with MoltenVK downloaded at a pinned version and checksum.

What players need: a GPU with Vulkan. Linux also wants the Vulkan loader and the GPU's driver,
ALSA, and X11 or Wayland with libxkbcommon. The archives run on Ubuntu 22.04's glibc or newer,
Windows 10 or newer (x64), macOS 12 or newer.

Saves, settings and logs go to the player's own folders (`%APPDATA%\Weft`,
`~/.local/share/weft`, `~/Library/Application Support/Weft`), not into the unpacked folder.

## macOS: unsigned, and signing

Without signing secrets the macOS archives are unsigned. macOS stops them the first time, and
the player either runs `xattr -dr com.apple.quarantine chiacraft` or tries once and then clicks
"Open Anyway" in System Settings > Privacy & Security (macOS 15 dropped the Control-click > Open
way round). `README.txt` in the archive says so.

With the secrets below, a `sign-macos` job runs after the builds, on a Mac runner, for each Mac
archive:

1. imports the certificate into a keychain of its own (deleted at the end);
2. `tools/release/macos-sign.sh`: signs every Mach-O file with the hardened runtime and a
   timestamp, which includes MoltenVK and each mod's library inside `run/mods/*.zip` (Apple's
   notary looks into nested archives). The game gets `tools/release/macos/entitlements.plist`:
   library validation off (players' mods are other people's native libraries), and JIT memory
   for wasmtime (the sandboxed mods);
3. sends it to Apple's notary and waits for "Accepted";
4. uploads `<platform>-signed`, which replaces the unsigned archive in a tag's release.

A bare executable can't carry a stapled ticket (only an `.app`, `.dmg` or `.pkg` can), so the
first start checks the notarization online. Making the game an `.app` bundle would allow stapling
and a Finder icon; that needs the game to find `assets/` and `run/` inside the bundle, an engine
change left for later.

Without secrets the job doesn't start at all, so no Mac runner is spent on it.

### The secrets Ronald adds

GitHub > the repository > Settings > Secrets and variables > Actions > New repository secret, or
with `gh secret set <NAME> -R ronickg/chiacraft-weft`, which asks for the value without echoing
it. No one else handles these.

| secret | what | needed for |
|---|---|---|
| `MACOS_CERT_P12` | the Developer ID Application certificate with its private key, exported as `.p12`, base64 | signing |
| `MACOS_CERT_PASSWORD` | the password chosen when exporting the `.p12` | signing |
| `APPLE_TEAM_ID` | the 10-character team ID (developer.apple.com > Account > Membership details) | picks the certificate; Apple ID notarizing |
| `APPLE_API_KEY_P8` | the text of the App Store Connect API key file `AuthKey_<id>.p8` | notarizing (way A, recommended) |
| `APPLE_API_KEY_ID` | that key's ID | way A |
| `APPLE_API_ISSUER_ID` | the issuer ID shown above the keys list | way A |
| `APPLE_ID` | the Apple ID's email | notarizing (way B) |
| `APPLE_APP_PASSWORD` | an app-specific password for that Apple ID | way B |

Only the first two: signed but not notarized (Gatekeeper still warns on other Macs). Add way A or
way B for builds that open cleanly. With both, way A is used.

### Getting the certificate (once)

1. See whether one exists: `security find-identity -v -p codesigning | grep "Developer ID Application"`.
   (The "Apple Development" certificate that `tools/macos-sign-run.sh` uses for local runs is a
   different one and can't be notarized.)
2. If none: Xcode > Settings > Accounts > the team > Manage Certificates > + > "Developer ID
   Application". Only the account holder can make one.
3. Keychain Access > login > My Certificates > right-click "Developer ID Application: … (TEAMID)"
   (with its key under it) > Export > `.p12`, with a new password.
4. Into the secret, then delete the file:

   ```
   base64 -i DeveloperID.p12 | gh secret set MACOS_CERT_P12 -R ronickg/chiacraft-weft
   gh secret set MACOS_CERT_PASSWORD -R ronickg/chiacraft-weft
   gh secret set APPLE_TEAM_ID -R ronickg/chiacraft-weft
   rm DeveloperID.p12
   ```

### The notary key (way A, once)

App Store Connect > Users and Access > Integrations > App Store Connect API > Team Keys > +, role
"Developer". Download `AuthKey_<id>.p8` (it can be downloaded only once), then:

```
gh secret set APPLE_API_KEY_P8 -R ronickg/chiacraft-weft < AuthKey_XXXXXXXXXX.p8
gh secret set APPLE_API_KEY_ID -R ronickg/chiacraft-weft
gh secret set APPLE_API_ISSUER_ID -R ronickg/chiacraft-weft
```

Way B instead: account.apple.com > Sign-In and Security > App-Specific Passwords, then set
`APPLE_ID` and `APPLE_APP_PASSWORD`.

Signing on his own Mac works with the same script, with the certificate in the login keychain:
`SIGN_IDENTITY="Developer ID Application: …" NOTARY_PROFILE=<profile> tools/release/macos-sign.sh in.tar.gz out.tar.gz`,
after `xcrun notarytool store-credentials <profile>` once.

## Windows: signing (not set up)

The Windows zip is unsigned, so SmartScreen says "Windows protected your PC" until the player
clicks "More info" > "Run anyway". The ways to sign, if it's ever wanted:

- **Azure Trusted Signing** (Microsoft's, about $10 a month): no hardware key, signs from
  GitHub Actions with `azure/trusted-signing-action`. Needs an identity check of the person or
  company; open to individuals in some countries only.
- **An OV or EV code signing certificate** from a CA (a few hundred dollars a year). Since 2023
  their keys must live on a hardware token or a cloud HSM, so CI signs through the CA's cloud
  service (e.g. DigiCert KeyLocker, SSL.com eSigner).
- **SignPath Foundation**: free for open-source projects, which this isn't yet.

Any of them signs `weft.exe` (and the mods' `.dll`s, like the macOS job) with `signtool` and a
timestamp. SmartScreen's warning fades as a signed program gains reputation; EV certificates no
longer skip that.
