import { execSync } from 'child_process';

/**
 * Repository identity.
 *
 * The repository's `owner/name` is not stable — the project is mid-rename to
 * "Media-Sort", and any future rename or transfer changes it again. The numeric
 * ID never changes, so it is the source of truth here and the current
 * `owner/name` is resolved from it at build time. Everything that needs a link
 * or a `gh --repo` argument must use these resolved values rather than a
 * hardcoded name (which is how `repoName = 'Media-Sort'` used to 404).
 *
 * Resolution is best-effort: without a usable `gh` the fallback below is used so
 * a plain `astro dev` still renders. The fallback is deliberately the *old*
 * name, because GitHub permanently redirects a renamed repository whereas a
 * not-yet-existing name simply does not resolve.
 */
export const repoId = 119281525;

const FALLBACK_REPO_FULL = 'Lolle2000la/Image-Sort';

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
