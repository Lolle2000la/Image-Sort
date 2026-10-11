#!/usr/bin/env bash
#
# Cut a release: bump the workspace version, refresh Cargo.lock, commit, and
# create the signed annotated tag that drives .github/workflows/release.yml.
#
# The tag is created HERE (not in CI) so it is signed with your own key
# (commit.gpgsign / tag.gpgsign), and its message is the prefacing release
# summary that the publish job prepends to the generated changelog. Pushing
# the tag is what finally triggers the release workflow.
#
# Usage:
#   scripts/release.sh <version> [-m <message> | -F <notes-file>] [options]
#
# Examples:
#   scripts/release.sh 3.0.0-alpha.18 -F /tmp/notes.md
#   scripts/release.sh 3.0.0 -m "First stable release of the Rust rewrite."
#   scripts/release.sh 3.0.0-alpha.19 --dry-run
#
# Options:
#   -m, --message <text>    Signed tag message (the release summary).
#   -F, --notes-file <file> Read the tag message from a file.
#       --require-ci        Fail unless every Actions run for HEAD succeeded.
#       --no-push           Stop after creating the tag; push it yourself.
#       --dry-run           Validate and print the plan; change nothing.
#       --no-verify         Skip the CI status check entirely.
#   -h, --help              Show this help.
#
# The version is the single source of truth: it is written to
# [workspace.package].version in Cargo.toml and mirrored into Cargo.lock, and
# release.yml refuses to build a tag whose name (minus the leading "v") does
# not match the manifest.

set -euo pipefail

VERSION=""
TAG_MESSAGE=""
NOTES_FILE=""
REQUIRE_CI=0
NO_PUSH=0
DRY_RUN=0
NO_VERIFY=0

die() { printf 'error: %s\n' "$*" >&2; exit 1; }
info() { printf '==> %s\n' "$*" >&2; }
warn() { printf 'warning: %s\n' "$*" >&2; }

usage() {
  cat <<'EOF'
Cut a release: bump the workspace version, refresh Cargo.lock, commit, and
create the signed annotated tag that drives .github/workflows/release.yml.

Usage:
  scripts/release.sh <version> [-m <message> | -F <notes-file>] [options]

Examples:
  scripts/release.sh 3.0.0-alpha.18 -F /tmp/notes.md
  scripts/release.sh 3.0.0 -m "First stable release of the Rust rewrite."
  scripts/release.sh 3.0.0-alpha.19 --dry-run

Options:
  -m, --message <text>    Signed tag message (the release summary).
  -F, --notes-file <file> Read the tag message from a file.
      --require-ci        Fail unless every Actions run for HEAD succeeded.
      --no-push           Stop after creating the tag; push it yourself.
      --dry-run           Validate and print the plan; change nothing.
      --no-verify         Skip the CI status check entirely.
  -h, --help              Show this help.
EOF
  exit "${1:-0}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    -m|--message)
      [ $# -ge 2 ] || die "$1 requires an argument"
      TAG_MESSAGE="$2"; shift 2 ;;
    -F|--notes-file)
      [ $# -ge 2 ] || die "$1 requires an argument"
      NOTES_FILE="$2"; shift 2 ;;
    --require-ci) REQUIRE_CI=1; shift ;;
    --no-push) NO_PUSH=1; shift ;;
    --dry-run) DRY_RUN=1; shift ;;
    --no-verify) NO_VERIFY=1; shift ;;
    -h|--help) usage 0 ;;
    -*) die "unknown option: $1 (try --help)" ;;
    *)
      [ -z "$VERSION" ] || die "version given twice: '$VERSION' and '$1'"
      VERSION="$1"; shift ;;
  esac
done

[ -n "$VERSION" ] || { printf 'error: missing <version>\n\n' >&2; usage 1; }
[ -z "$TAG_MESSAGE" ] || [ -z "$NOTES_FILE" ] || die "-m and -F are mutually exclusive"

VERSION="${VERSION#v}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  die "invalid version '$VERSION' (expected e.g. 3.0.0 or 3.0.0-alpha.18)"
fi
TAG="v$VERSION"

command -v cargo >/dev/null || die "cargo not found on PATH"

cd "$(git rev-parse --show-toplevel)" || die "not inside a git repository"

# --- preflight -------------------------------------------------------------

info "preflight checks"

git rev-parse --verify -q HEAD >/dev/null || die "repository has no commits"

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" != "HEAD" ] || die "refusing to release from a detached HEAD"

if [ -n "$(git status --porcelain)" ]; then
  die "working tree is not clean; commit or stash first:
$(git status --short)"
fi

CURRENT_VERSION="$(awk '
  /^\[workspace\.package\]/ { in_section = 1; next }
  /^\[/ { in_section = 0 }
  in_section && /^version[[:space:]]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }
' Cargo.toml)"
[ -n "$CURRENT_VERSION" ] || die "could not read [workspace.package].version from Cargo.toml"
[ "$VERSION" != "$CURRENT_VERSION" ] || die "Cargo.toml is already at version $VERSION"

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  die "tag $TAG already exists locally"
fi
git fetch --quiet --tags origin || warn "could not fetch from origin; continuing"
if git ls-remote --tags --exit-code origin "refs/tags/$TAG" >/dev/null 2>&1; then
  die "tag $TAG already exists on origin"
fi

if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
  read -r BEHIND AHEAD < <(git rev-list --left-right --count '@{u}...HEAD')
  [ "$BEHIND" -eq 0 ] || die "local $BRANCH is $BEHIND commit(s) behind its upstream; pull first"
  [ "$AHEAD" -eq 0 ] || warn "local $BRANCH is $AHEAD commit(s) ahead of upstream (they will be pushed too)"
else
  warn "branch $BRANCH has no upstream configured"
fi

check_ci() {
  local head_sha runs_json bad
  head_sha="$(git rev-parse HEAD)"
  if ! command -v gh >/dev/null 2>&1; then
    warn "gh not found; skipping CI status check"
    return 0
  fi
  if ! command -v jq >/dev/null 2>&1; then
    warn "jq not found; skipping CI status check"
    return 0
  fi
  runs_json="$(gh run list --commit "$head_sha" --limit 100 \
    --json name,status,conclusion 2>/dev/null || echo '[]')"
  if [ "$runs_json" = "[]" ] || [ -z "$runs_json" ]; then
    if [ "$REQUIRE_CI" -eq 1 ]; then
      die "no Actions runs found for HEAD ($head_sha)"
    fi
    warn "no Actions runs found for HEAD ($head_sha); skipping CI check"
    return 0
  fi
  bad="$(printf '%s' "$runs_json" | jq -r '
    [ .[] | select(
        .status != "completed"
        or (.conclusion | IN("success", "skipped", "neutral") | not)
      ) | "  \(.name): \(.status)/\(.conclusion)" ] | join("\n")')"
  if [ -n "$bad" ]; then
    if [ "$REQUIRE_CI" -eq 1 ]; then
      die "CI is not green for HEAD ($head_sha):
$bad"
    fi
    warn "CI is not green for HEAD ($head_sha):
$bad"
  else
    info "all Actions runs for HEAD succeeded"
  fi
}

if [ "$NO_VERIFY" -eq 0 ]; then
  check_ci
fi

# --- plan ------------------------------------------------------------------

if [ -n "$NOTES_FILE" ]; then
  [ -f "$NOTES_FILE" ] || die "notes file not found: $NOTES_FILE"
  TAG_ARGS=(-F "$NOTES_FILE")
elif [ -n "$TAG_MESSAGE" ]; then
  TAG_ARGS=(-m "$TAG_MESSAGE")
else
  warn "no -m/-F given; using a default tag message"
  TAG_ARGS=(-m "Release $TAG")
fi

info "version: $CURRENT_VERSION -> $VERSION"
info "branch:  $BRANCH"
info "tag:     $TAG (signed)"

if [ "$DRY_RUN" -eq 1 ]; then
  info "dry run: no changes made"
  exit 0
fi

# --- bump ------------------------------------------------------------------

BUMPED=0
COMMITTED=0

restore() {
  local code=$?
  if [ "$code" -ne 0 ] && [ "$COMMITTED" -eq 0 ] && [ "$BUMPED" -eq 1 ]; then
    warn "rolling back Cargo.toml and Cargo.lock"
    git checkout -- Cargo.toml Cargo.lock 2>/dev/null || true
  fi
  return "$code"
}
trap restore EXIT

info "bumping Cargo.toml"
CARGO_TMP="$(mktemp)"
awk -v new="$VERSION" '
  /^\[workspace\.package\]/ { in_section = 1; print; next }
  /^\[/ { in_section = 0 }
  in_section && !done && /^version[[:space:]]*=/ { sub(/=.*/, "= \"" new "\""); done = 1 }
  { print }
' Cargo.toml > "$CARGO_TMP"
grep -q "^version = \"$VERSION\"\$" "$CARGO_TMP" || { rm -f "$CARGO_TMP"; die "failed to rewrite the workspace version"; }
mv "$CARGO_TMP" Cargo.toml
BUMPED=1

info "refreshing Cargo.lock"
cargo update --workspace >/dev/null 2>&1 || die "cargo update --workspace failed"
cargo metadata --locked --format-version 1 >/dev/null || die "Cargo.lock is inconsistent after the bump"
LOCK_VERSION="$(grep -A1 '^name = "media-sort-gui"$' Cargo.lock | grep '^version = ' | head -1 | cut -d'"' -f2)"
[ "$LOCK_VERSION" = "$VERSION" ] || die "Cargo.lock still has media-sort-gui $LOCK_VERSION"

info "committing"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION"
COMMITTED=1

info "creating signed tag $TAG"
git tag -s "$TAG" "${TAG_ARGS[@]}" || {
  die "failed to create the signed tag; is tag signing configured (tag.gpgsign / user.signingkey)?"
}

if [ "$NO_PUSH" -eq 1 ]; then
  info "skipping push (--no-push); finish with:"
  printf '    git push origin %s\n    git push origin refs/tags/%s\n' "$BRANCH" "$TAG" >&2
  exit 0
fi

info "pushing $BRANCH"
git push origin "HEAD:refs/heads/$BRANCH"
info "pushing tag $TAG (this triggers the release workflow)"
git push origin "refs/tags/$TAG"

info "done. $TAG pushed; the release workflow is building it now."
cat >&2 <<EOF

    gh run watch \$(gh run list --workflow=release.yml --limit=1 --json databaseId --jq '.[0].databaseId')
EOF
