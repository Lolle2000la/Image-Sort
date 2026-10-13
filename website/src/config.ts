import { execSync } from 'child_process';

/**
 * Repository identity.
 *
 * The repository's `owner/name` is not stable — it was renamed to "Media-Sort"
 * and any future rename or transfer changes it again. The numeric ID never
 * changes, so it is the source of truth here and the current `owner/name` is
 * resolved from it at build time. Everything that needs a link or a `gh --repo`
 * argument must use these resolved values rather than a hardcoded name (which is
 * how `repoName = 'Media-Sort'` used to 404 before the rename).
 *
 * Resolution is best-effort: without a usable `gh` the fallback below is used so
 * a plain `astro dev` still renders. It holds the current name; a pre-rename
 * `Image-Sort` link keeps working too, because GitHub permanently redirects a
 * renamed repository.
 */
export const repoId = 119281525;

const FALLBACK_REPO_FULL = 'Lolle2000la/Media-Sort';

function resolveRepoFull(): string {
    try {
        // `GET /repositories/{id}` works for any visibility and returns the
        // name the repository currently has.
        const resolved = execSync(
            `gh api /repositories/${repoId} --jq .full_name`,
            { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }
        ).trim();
        return resolved || FALLBACK_REPO_FULL;
    } catch {
        return FALLBACK_REPO_FULL;
    }
}

export const repoFull = resolveRepoFull();
export const repoUrl = `https://github.com/${repoFull}`;

const [repoOwner = '', repoName = ''] = repoFull.split('/');
export { repoName, repoOwner };

/**
 * This snap's Snap Store listing. The snap is distributed through the store
 * rather than as a file, so links point here (the release also attaches a
 * `.snap` for offline installs; see the installation page).
 */
export const snapStoreUrl = 'https://snapcraft.io/media-sort';
