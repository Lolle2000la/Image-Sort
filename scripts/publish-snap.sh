#!/usr/bin/env bash
#
# Attach the built .snap files to the GitHub release and publish them to the
# Snap Store.
#
# Shared by the two snap backends in release.yml:
#   * `snap`           - builds natively on the matching GitHub runner
#   * `snap-launchpad` - builds every architecture on Launchpad
# Both end up with .snap files in a directory and identical expectations from
# then on: the release carries them (that is the offline install path) and the
# Store gets the revision, on a channel chosen from the tag.
#
# Usage: scripts/publish-snap.sh <directory-with-snaps>
#
# Environment:
#   TAG                         release tag (github.ref_name)
#   GH_TOKEN                    token used by `gh release upload`
#   SNAPCRAFT_STORE_CREDENTIALS exported login (snapcraft export-login); when
#                               unset the Store publish is skipped
#
# The caller runs this with `continue-on-error: true`: the GitHub release is
# complete and valid without a Store revision, so a Store-side problem must not
# block it, but it must stay visible instead of silently succeeding.
set -euo pipefail

dist_dir="${1:?usage: publish-snap.sh <directory-with-snaps>}"
tag="${TAG:?TAG is required}"

shopt -s nullglob
snaps=("$dist_dir"/*.snap)
if [ "${#snaps[@]}" -eq 0 ]; then
  echo "::warning::no .snap found in $dist_dir; nothing to attach or publish"
  exit 0
fi

echo "Attaching ${#snaps[@]} snap(s) to release $tag:"
printf '  %s\n' "${snaps[@]}"
gh release upload "$tag" "${snaps[@]}" --clobber

if [ -z "${SNAPCRAFT_STORE_CREDENTIALS:-}" ]; then
  echo "::warning::SNAPCRAFT_STORE_CREDENTIALS is not set; skipping Snap Store publish"
  exit 0
fi

# Stable tags go to latest/stable; a prerelease tag (contains '-') goes to
# latest/candidate so it does not reach stable users.
case "$tag" in
  *-*) channel="latest/candidate" ;;
  *) channel="latest/stable" ;;
esac

for snap in "${snaps[@]}"; do
  echo "Publishing $snap to $channel"
  snapcraft upload --release="$channel" "$snap"
done
