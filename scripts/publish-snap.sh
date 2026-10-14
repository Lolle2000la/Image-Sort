#!/usr/bin/env bash
#
# Publish built .snap files to the Snap Store, on the channel the release
# implies.
#
# Usage: scripts/publish-snap.sh <directory-with-snaps>
#
# Environment:
#   TAG                         release tag (`github.ref_name`). Used to pick the
#                               channel when REQUESTED_CHANNEL is unset: a
#                               stable tag goes to latest/stable, a prerelease
#                               tag (one containing '-') to latest/candidate.
#                               Required in that case, and must look like a
#                               release tag -- dispatching this on a branch is a
#                               mistake, not a way to reach stable.
#   REQUESTED_CHANNEL           explicit channel risk (`stable`, `candidate`,
#                               `beta` or `edge`), or `auto`; overrides TAG.
#                               Set by the workflow_dispatch input.
#   SNAPCRAFT_STORE_CREDENTIALS exported store login (snapcraft export-login).
#                               Required: this script exists to publish, so a
#                               missing credential is an error, not a no-op.
#
# The revisions are uploaded per architecture; the Store keeps one revision per
# architecture and channel, so a re-run of an unchanged build is a no-op on the
# Store side.
set -euo pipefail

dist_dir="${1:?usage: publish-snap.sh <directory-with-snaps>}"

shopt -s nullglob
snaps=("$dist_dir"/*.snap)
if [ "${#snaps[@]}" -eq 0 ]; then
  echo "::error::no .snap files in $dist_dir"
  exit 1
fi

if [ -n "${REQUESTED_CHANNEL:-}" ] && [ "$REQUESTED_CHANNEL" != "auto" ]; then
  risk="$REQUESTED_CHANNEL"
else
  tag="${TAG:?TAG is required when no channel is requested}"
  case "$tag" in
    v*) ;;
    *)
      echo "::error::ref '$tag' is not a release tag, so there is no channel to infer."
      echo "Dispatch this on a v* tag, or pick a channel explicitly."
      exit 1
      ;;
  esac
  case "$tag" in
    *-*) risk="candidate" ;;
    *) risk="stable" ;;
  esac
fi

case "$risk" in
  stable | candidate | beta | edge) ;;
  *)
    echo "::error::unknown channel '$risk'; expected stable, candidate, beta or edge."
    exit 1
    ;;
esac
channel="latest/$risk"

if [ -z "${SNAPCRAFT_STORE_CREDENTIALS:-}" ]; then
  cat >&2 <<'MSG'
::error::SNAPCRAFT_STORE_CREDENTIALS is not set, so nothing can be published.
Export a login for this snap and store it on the `release` environment:

  snapcraft export-login --snaps media-sort \
    --channels latest/stable,latest/candidate,latest/beta,latest/edge \
    /tmp/media-sort-store-login.txt
  gh secret set SNAPCRAFT_STORE_CREDENTIALS --env release < /tmp/media-sort-store-login.txt

See AGENTS.md ("Snap packaging" -> "Store credentials") for the details.
MSG
  exit 1
fi

echo "Publishing ${#snaps[@]} revision(s) to $channel:"
printf '  %s\n' "${snaps[@]}"
for snap in "${snaps[@]}"; do
  echo "--- $snap"
  snapcraft upload --release="$channel" "$snap"
done

# Best effort: the upload output is the release record, this is the confirmation.
echo "Channel state after publishing:"
snapcraft status media-sort || echo "(could not read the channel state; check https://snapcraft.io/media-sort)"
