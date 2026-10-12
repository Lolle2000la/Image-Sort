#!/usr/bin/env bash
#
# Generate the "Downloads" section that release.yml prepends to the published
# release notes.
#
# Velopack uploads a dozen-plus assets per platform (full and delta .nupkg
# packages, their .sig files, update manifests, portable archives, ...), so the
# Assets list on the GitHub release page buries the handful of files a user
# actually wants to install. This script turns the release's asset list into a
# small markdown table that points straight at the installers.
#
# Input: TSV lines "asset-name<TAB>download-url" on stdin, i.e. exactly what
#   gh api "repos/<owner>/<repo>/releases/tags/<tag>" \
#     --jq '.assets[] | [.name, .browser_download_url] | @tsv'
# produces for the release.
#
# Output: markdown on stdout, or nothing at all when no known installer asset
# is present (the caller keeps the notes unchanged in that case).

set -euo pipefail

declare -A ASSET_URL=()
while IFS=$'\t' read -r name url; do
  [ -n "${name:-}" ] || continue
  ASSET_URL["$name"]="$url"
done

# link <name-regex> <label>: print a markdown link to the first matching asset,
# or nothing when the asset is absent (e.g. a platform's build was skipped).
link() {
  local pattern="$1" label="$2" name
  # An empty pattern would match every asset, so treat it as "no such slot".
  [ -n "$pattern" ] || return 0
  for name in "${!ASSET_URL[@]}"; do
    if [[ "$name" =~ $pattern ]]; then
      printf '[%s](%s)' "$label" "${ASSET_URL[$name]}"
      return 0
    fi
  done
}

# row <platform> <installer-regex> <installer-label> <portable-regex> <portable-label>
# Emits a table row, or nothing when neither asset exists.
row() {
  local platform="$1" ipat="$2" ilabel="$3" ppat="$4" plabel="$5"
  local installer portable
  installer="$(link "$ipat" "$ilabel")"
  portable="$(link "$ppat" "$plabel")"
  [ -n "$installer$portable" ] || return 0
  printf '| %s | %s | %s |\n' "$platform" "${installer:--}" "${portable:--}"
}

# The asset names are Velopack's "<AppId>-<channel>-<kind>" scheme, where the
# channel is the platform passed to `vpk pack` (win-x64, win-arm64, osx-arm64,
# linux-x64, linux-arm64). Match on a suffix so a version prefix, if Velopack
# ever adds one, does not break the lookup.
TABLE="$({
  row 'Windows (x64)'         'win-x64-Setup\.exe$'    'Setup (.exe)'  'win-x64-Portable\.zip$'    'Portable (.zip)'
  row 'Windows (ARM64)'       'win-arm64-Setup\.exe$'  'Setup (.exe)'  'win-arm64-Portable\.zip$'  'Portable (.zip)'
  row 'macOS (Apple Silicon)' 'osx-arm64-Setup\.pkg$'  'Setup (.pkg)'  'osx-arm64-Portable\.zip$'  'Portable (.zip)'
  row 'Linux (x64)'           'linux-x64\.AppImage$'   'AppImage'      ''                          ''
  row 'Linux (ARM64)'         'linux-arm64\.AppImage$' 'AppImage'      ''                          ''
  # Snap Store is the primary Linux distribution channel, but the .snap is also
  # attached to the release so it can be installed offline / via `snap install
  # --classic`. Snapcraft names assets "<name>_<version>_<arch>.snap".
  row 'Linux (x64)'           '_amd64\.snap$'          'Snap'          ''                          ''
  row 'Linux (ARM64)'         '_arm64\.snap$'          'Snap'          ''                          ''
} )"

[ -n "$TABLE" ] || exit 0

cat <<EOF
## Downloads

Grab the installer for your platform below. Everything else under **Assets** is
package/update data that the app's built-in updater consumes automatically and
is not meant to be downloaded by hand.

| Platform | Installer | Portable |
| --- | --- | --- |
$TABLE

EOF
