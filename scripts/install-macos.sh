#!/bin/bash
# Install a built .app without requiring Node, Rust, Homebrew, or sudo.
set -euo pipefail

fail() { printf '%s\n' "$*" >&2; exit 1; }
[[ "$(uname -s)" == Darwin ]] || fail 'This installer requires macOS.'
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
app_source="$script_dir/Hourglass.app"
if [[ ! -d "$app_source" ]]; then
  app_source="$script_dir/../src-tauri/target/release/bundle/macos/Hourglass.app"
fi
app_dir=/Applications
start_app=true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) [[ $# -ge 2 ]] || fail '--app requires a path'; app_source="$2"; shift 2 ;;
    --appdir) [[ $# -ge 2 && -n "$2" ]] || fail '--appdir requires a path'; app_dir="$2"; shift 2 ;;
    --no-start) start_app=false; shift ;;
    --help) printf '%s\n' 'Usage: install-macos.sh [--app PATH] [--appdir DIRECTORY] [--no-start]'; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done
[[ -d "$app_source" ]] || fail 'Hourglass.app was not found. Run npm run build -- --bundles app first.'
bundle_id=$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$app_source/Contents/Info.plist")
[[ "$bundle_id" == io.hourglass.timer ]] || fail 'The source is not a Hourglass app bundle.'
[[ -x "$app_source/Contents/MacOS/hourglass" ]] || fail 'The Hourglass executable is missing.'

if [[ "$start_app" == true ]] && /usr/bin/pgrep -x hourglass >/dev/null 2>&1; then
  fail 'Quit the running Hourglass from its settings before installing. Then run this installer again.'
fi
if [[ "$app_dir" == /Applications && ! -w "$app_dir" ]]; then
  app_dir="$HOME/Applications"
fi
mkdir -p -- "$app_dir"
app_dir="$(cd -- "$app_dir" && pwd)"
destination="$app_dir/Hourglass.app"
[[ "$(cd -- "$app_source" && pwd)" != "$destination" ]] || fail 'The source app is already at the installation destination.'
if [[ -e "$destination" || -L "$destination" ]]; then
  [[ ! -L "$destination" ]] || fail 'Destination is a symlink. Manage this installation through its package manager.'
  existing_id=$(/usr/libexec/PlistBuddy -c 'Print CFBundleIdentifier' "$destination/Contents/Info.plist")
  [[ "$existing_id" == io.hourglass.timer ]] || fail 'Another app already occupies the destination.'
fi

stage="$(mktemp -d "$app_dir/.hourglass-install.XXXXXX")"
committed=false
cleanup() {
  if [[ "$committed" == false && -d "$stage/previous.app" ]]; then
    [[ ! -e "$destination" ]] || rm -rf -- "$destination"
    mv -- "$stage/previous.app" "$destination"
  fi
  rm -rf -- "$stage"
}
trap cleanup EXIT
/usr/bin/ditto "$app_source" "$stage/Hourglass.app"
if [[ -d "$destination" ]]; then mv -- "$destination" "$stage/previous.app"; fi
mv -- "$stage/Hourglass.app" "$destination"
committed=true
printf 'Installed: %s\n' "$destination"
if [[ "$start_app" == true ]]; then
  /usr/bin/open "$destination"
  printf '%s\n' 'Hourglass is in the menu bar. Launch at login is enabled on its first installed launch.'
else
  printf '%s\n' 'Open Hourglass.app to finish launch-at-login registration.'
fi
