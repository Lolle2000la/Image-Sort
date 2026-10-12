#!/usr/bin/env bash
#
# Build the Media Sort snap in one of its two confinement variants.
#
#   strict  (default)  store-publishable build       -> latest/stable
#   classic            full filesystem access        -> latest/classicmode
#                      (needs the store's classic-confinement approval)
#
# Both variants come from the single snap/snapcraft.yaml: the classic one only
# swaps `confinement` and drops the app's `plugs` block. snapcraft resolves
# exactly one snap/snapcraft.yaml per project directory, so rather than keeping
# a second, drifting copy of the ~50-line parts section, the variant is
# materialised into a throwaway project whose part points `source` back at this
# repository. The derivation is guarded: if snapcraft.yaml is restructured so
# the patterns below no longer match, the build fails loudly instead of
# silently producing the wrong variant.
#
# Usage:
#   scripts/build-snap.sh [strict|classic] [snapcraft pack args...]
#
# Examples:
#   scripts/build-snap.sh strict                      # builds via LXD
#   scripts/build-snap.sh classic --use-lxd
#   scripts/build-snap.sh strict --destructive-mode   # Ubuntu 24.04 host / CI
#
# `--use-lxd` requires the `lxd` group (e.g. `sg lxd -c 'bash scripts/build-snap.sh'`).

set -euo pipefail

variant="${1:-strict}"
shift || true
case "$variant" in
  strict | classic) ;;
  *)
    echo "usage: $0 [strict|classic] [snapcraft pack args...]" >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src_yaml="$repo_root/snap/snapcraft.yaml"
start_dir="$(pwd)"
project="$(mktemp -d "${TMPDIR:-/tmp}/media-sort-snap-XXXXXX")"
trap 'rm -rf "$project"' EXIT

# Guards: fail loudly if snapcraft.yaml drifts away from what this script edits.
grep -qx 'confinement: strict' "$src_yaml" ||
  { echo "error: $src_yaml has no top-level 'confinement: strict' to switch" >&2; exit 1; }
grep -qx '    plugs:' "$src_yaml" ||
  { echo "error: $src_yaml has no app 'plugs:' block to drop for classic" >&2; exit 1; }
grep -qx '    source: \.' "$src_yaml" ||
  { echo "error: $src_yaml has no '    source: .' to repoint at the repo" >&2; exit 1; }

mkdir -p "$project/snap"
cp -a "$repo_root/snap/gui" "$project/snap/gui"

# The throwaway project only holds the yaml and the GUI assets, so repoint the
# part's source at the real working tree.
sed "s|^    source: \.$|    source: $repo_root|" "$src_yaml" >"$project/snap/snapcraft.yaml"

if [ "$variant" = classic ]; then
  # Drop the `plugs:` block (a classic snap is unconfined) and switch
  # confinement. The awk skips the key, its indented items and their comments,
  # and keeps everything else byte-for-byte.
  awk '
    skip_plugs {
      if ($0 ~ /^      - / || $0 ~ /^      #/) next
      skip_plugs = 0
    }
    /^    plugs:$/ { skip_plugs = 1; next }
    { print }
  ' "$project/snap/snapcraft.yaml" >"$project/snap/snapcraft.yaml.new"
  mv "$project/snap/snapcraft.yaml.new" "$project/snap/snapcraft.yaml"
  sed -i 's/^confinement: strict$/confinement: classic/' "$project/snap/snapcraft.yaml"
fi

echo "==> building '$variant' snap (project: $project)"
cd "$project"
snapcraft pack "$@"

mapfile -t snaps < <(find "$project" -name '*.snap' -type f)
[ "${#snaps[@]}" -gt 0 ] ||
  { echo "error: snapcraft produced no .snap" >&2; exit 1; }
for snap in "${snaps[@]}"; do
  # Both variants share name/version/arch, so the classic artefact (and any
  # non-default variant) is suffixed to keep the two installers distinct as
  # release assets; the snap's *internal* name/version are unchanged.
  base="$(basename "$snap")"
  if [ "$variant" = strict ]; then
    out="$base"
  else
    out="${base%.snap}-$variant.snap"
  fi
  cp -v "$snap" "$start_dir/$out"
done
