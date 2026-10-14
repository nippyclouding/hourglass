# Publishing Hourglass

Repository: [nippyclouding/hourglass](https://github.com/nippyclouding/hourglass).

The [v0.1.0 release](https://github.com/nippyclouding/hourglass/releases/tag/v0.1.0) is public. Universal macOS and Windows x64 installers passed native CI; the published archive, generated Cask, Git source installation, and Homebrew installation have been verified. Login registration points to the Applications executable. UI interaction and actual login/Sleep tests remain pending. See BUILD_STATUS.md for results and signing limitations.

## Local macOS app and installer

```sh
npm ci
npm run package:macos
```

`dist/` contains a signed local `Hourglass.app`, a double-click `Install Hourglass.command`, an architecture-specific ZIP, and its SHA-256 checksum. The ZIP contains the app, installer, and MIT license. Run the installer beside the app; it copies the bundle into `/Applications`, or `~/Applications` when the system folder is not writable, and opens it. Quit an existing Hourglass before installing an update.

The first installed release launch enables login startup through the official Tauri autostart plugin. Development/build/download paths do not register macOS login startup. An initialization marker prevents subsequent launches from re-enabling a setting the user has turned off. Login starts the app in its idle state; it does not start a timer.

The local installer supports `--app PATH`, `--appdir DIRECTORY`, and `--no-start`. The last option copies the app without launching or registering startup. Automated integration tests use temporary directories and this option; they do not modify Applications or login items.

## GitHub Releases

1. Push this project, including both lockfiles and `.github/workflows/`, to the repository.
2. Keep versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` aligned.
3. Tag the release and push the tag. For the current version:

```sh
git tag v0.1.0
git push origin v0.1.0
```

`release.yml` runs checks, builds a **Universal macOS app** (Apple Silicon and Intel) and a **Windows x64 NSIS installer**, and creates a **draft release**. The macOS job also uploads `Hourglass_0.1.0_universal.zip`, its checksum, and a generated `hourglass.rb` Cask. Inspect the draft assets before publishing the release.

When you publish the draft, `homebrew.yml` downloads the actual published Universal ZIP, computes its SHA-256, and commits `Casks/hourglass.rb` to the repository's default branch. This is an intentional automated repository change. The repository must allow GitHub Actions to write contents; default-branch protection can block this update. If it does, download the generated Cask from the release and submit it to the default branch using the normal review process.

Both workflows have run successfully for v0.1.0. The Universal macOS build has passed locally. Windows x64 NSIS compilation passed GitHub Actions; actual Windows runtime is still unverified. CI results are recorded in BUILD_STATUS.md.

## Homebrew installation

After the release is public and `Casks/hourglass.rb` is on the default branch:

```sh
brew tap nippyclouding/hourglass https://github.com/nippyclouding/hourglass.git
brew install --cask nippyclouding/hourglass/hourglass
open /Applications/Hourglass.app
```

The explicit tap URL uses the existing application repository, so a separate `homebrew-hourglass` repository is unnecessary. Follow Homebrew's trust prompt for this external tap. The Cask installs the `.app` without Node or Rust. Launching it once enables login startup. Users can turn that off in Hourglass settings.

```sh
brew upgrade --cask nippyclouding/hourglass/hourglass
brew uninstall --cask nippyclouding/hourglass/hourglass
```

Uninstalling stops the app and removes its `io.hourglass.timer` launch agent. A saved LaunchAgent is restored on an upgrade or reinstall so the existing login choice is preserved. Settings are preserved unless the user explicitly requests `brew uninstall --cask --zap nippyclouding/hourglass/hourglass`.

To regenerate a Cask manually, use the **published** archive, not a separately rebuilt ZIP:

```sh
npm run cask -- --archive dist/Hourglass_0.1.0_universal.zip
```

The generator uses the repository metadata in `package.json`; `--repo OWNER/REPO` can override it. It requires a real SHA-256 and never uses `:no_check`. Local Apple Silicon packaging cannot be labeled Universal: the packaging script checks both architectures with `lipo`.

See the [Homebrew Cask cookbook](https://docs.brew.sh/Cask-Cookbook) and [tap documentation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap) for the distribution format.

## macOS signing and notarization

Local builds use an **ad-hoc signature**, including the bundle resources. This is suitable for local builds and is not Apple Developer ID signing or notarization. The current artifact has not been notarized.

For public downloads that pass Gatekeeper without manual approval, configure these repository Actions secrets before tagging:

- `APPLE_CERTIFICATE`: base64-encoded Developer ID Application `.p12`.
- `APPLE_CERTIFICATE_PASSWORD`: exported certificate password.
- `APPLE_SIGNING_IDENTITY`: the Developer ID Application identity.
- `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`: notarization credentials; the password is an Apple app-specific password.

The workflow imports the certificate into a temporary CI keychain, builds with the supplied identity and notarization credentials, and removes the keychain afterward. Keep credentials in GitHub Secrets; the repository contains no signing credentials. See [Tauri's signing documentation](https://v2.tauri.app/distribute/sign/macos/).

Windows signing is not configured. Verify the generated Windows installer on a Windows computer before publishing it as a tested release.
