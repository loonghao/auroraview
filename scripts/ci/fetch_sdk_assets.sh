#!/usr/bin/env bash
# Fetch the AuroraView web SDK inject scripts from a pinned release of
# try-auroraview/auroraview-web-sdk and vendor them into the crate asset trees
# that rust-embed compiles into the binary.
#
# Replaces the previous `just sdk-build-assets` step, which built these scripts
# inside this monorepo. The SDK now lives in its own
# repository and ships a signed, versioned assets tarball per release.
set -euo pipefail

REPO="${AURORAVIEW_SDK_REPO:-try-auroraview/auroraview-web-sdk}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Pinned in .auroraview-sdk-version and advanced by the sync-sdk-assets workflow.
# AURORAVIEW_SDK_ASSETS_VERSION overrides the pin for local experiments.
VERSION="${AURORAVIEW_SDK_ASSETS_VERSION:-$(tr -d '[:space:]' < "$root/.auroraview-sdk-version" 2>/dev/null || true)}"
if [[ -z "$VERSION" ]]; then
  echo "Set AURORAVIEW_SDK_ASSETS_VERSION or commit .auroraview-sdk-version" >&2
  exit 1
fi
core_assets="$root/crates/auroraview-core/src/assets/js"
extensions_assets="$root/crates/auroraview-extensions/src/assets/js"

archive="sdk-assets-v${VERSION}.tar.gz"
# release-please derives the `sdk` component from the package name, so tags
# are `sdk-v<version>`.
url="https://github.com/${REPO}/releases/download/sdk-v${VERSION}/${archive}"
checksum_url="${url}.sha256"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

echo "Fetching SDK inject assets ${VERSION} from ${REPO}"
curl -fsSL --retry 3 --retry-delay 2 -o "$workdir/$archive" "$url"
curl -fsSL --retry 3 --retry-delay 2 -o "$workdir/$archive.sha256" "$checksum_url"

( cd "$workdir" && sha256sum --check --status "$archive.sha256" )
echo "Checksum verified for $archive"

mkdir -p "$workdir/extracted"
tar -xzf "$workdir/$archive" -C "$workdir/extracted"

for file in \
  core/event_bridge.js core/state_bridge.js core/bridge_stub.js \
  plugins/fs.js plugins/dialog.js plugins/clipboard.js plugins/shell.js \
  extensions/chrome_polyfill.js; do
  [[ -f "$workdir/extracted/$file" ]] || { echo "Missing $file in $archive" >&2; exit 1; }
done

install -d "$core_assets/core" "$core_assets/plugins" "$extensions_assets"
install -m 644 "$workdir/extracted"/core/*.js "$core_assets/core/"
install -m 644 "$workdir/extracted"/plugins/*.js "$core_assets/plugins/"
install -m 644 "$workdir/extracted/extensions/chrome_polyfill.js" "$extensions_assets/"

echo "Vendored SDK assets ${VERSION} into:"
echo "  $core_assets/core/"
echo "  $core_assets/plugins/"
echo "  $extensions_assets/"
