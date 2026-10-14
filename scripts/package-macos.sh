#!/bin/bash
set -euo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
project_dir="$(cd -- "$script_dir/.." && pwd)"
app_source="${1:-$project_dir/src-tauri/target/release/bundle/macos/Hourglass.app}"
arch="${2:-$(uname -m)}"
case "$arch" in arm64|x86_64|universal) ;; *) printf 'Unsupported architecture: %s\n' "$arch" >&2; exit 1 ;; esac
version="$(node -p 'require(process.argv[1]).version' "$project_dir/package.json")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { printf '%s\n' 'Expected a stable x.y.z version.' >&2; exit 1; }
[[ -x "$app_source/Contents/MacOS/hourglass" ]] || { printf '%s\n' 'Build Hourglass.app before packaging.' >&2; exit 1; }
if [[ "$arch" == universal ]]; then
  architectures="$(/usr/bin/lipo -archs "$app_source/Contents/MacOS/hourglass")"
  [[ " $architectures " == *" arm64 "* && " $architectures " == *" x86_64 "* ]] || {
    printf 'Universal packaging requires arm64 and x86_64, found: %s\n' "$architectures" >&2
    exit 1
  }
fi
mkdir -p "$project_dir/dist"
stage="$(mktemp -d "$project_dir/dist/.package.XXXXXX")"
trap 'rm -rf -- "$stage"' EXIT
/usr/bin/ditto "$app_source" "$stage/Hourglass.app"
cp "$script_dir/install-macos.sh" "$stage/Install Hourglass.command"
cp "$project_dir/LICENSE" "$stage/LICENSE"
chmod +x "$stage/Install Hourglass.command"
archive="Hourglass_${version}_${arch}.zip"
(cd "$stage" && /usr/bin/ditto -c -k --sequesterRsrc --keepParent Hourglass.app "$project_dir/dist/$archive")
# The installer is beside the app when the zip is extracted.
(cd "$stage" && /usr/bin/zip -q "$project_dir/dist/$archive" 'Install Hourglass.command' LICENSE)
(cd "$project_dir/dist" && /usr/bin/shasum -a 256 "$archive" > "$archive.sha256")
/usr/bin/ditto "$stage/Hourglass.app" "$project_dir/dist/Hourglass.app"
cp "$stage/Install Hourglass.command" "$project_dir/dist/Install Hourglass.command"
printf 'Packaged: %s\n' "$project_dir/dist/$archive"
