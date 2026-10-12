/**
 * Rewrite the repository's canonical name to its current one in Markdown/MDX
 * sources, before they are compiled.
 *
 * The docs are written against the repository's canonical name
 * (`Lolle2000la/Media-Sort`), which does not resolve until the rename lands.
 * `src/config.ts` resolves the real name from the repository's stable numeric
 * ID; this keeps link targets, visible link text and code snippets
 * (`git clone`) consistent with it. Doing it as a source transform rather than
 * a markdown plugin avoids pulling in Astro's legacy unified markdown pipeline
 * (remark/rehype plugins now require `@astrojs/markdown-remark`, which changes
 * how the whole site is rendered).
 *
 * The pattern is a fully qualified `owner/name`, so it cannot catch unrelated
 * prose. After the rename this is an identity mapping — harmless, and a safety
 * net if the repository is ever renamed again.
 */
export const CANONICAL_REPO = 'Lolle2000la/Media-Sort';

/** The repository's canonical *name* (`Media-Sort`), without the owner. */
const CANONICAL_REPO_NAME = CANONICAL_REPO.split('/')[1];

/**
 * @param {string} repoFull
 * @returns {import('vite').Plugin}
 */
export default function canonicalRepo(repoFull) {
    const repoName = repoFull.split('/')[1];
    // The fully qualified form first (so a transfer updates the owner too),
    // then the bare name, which is what the `cd <repo>` line of a clone snippet
    // uses. The hyphenated form is the repository identifier; the project's
    // *brand* is written with a space ("Media Sort") and is never touched.
    const rewrite = (text) =>
        text
            .split(CANONICAL_REPO)
            .join(repoFull)
            .split(CANONICAL_REPO_NAME)
            .join(repoName);

    return {
        name: 'media-sort:canonical-repo',
        enforce: 'pre',
        transform(code, id) {
            if (
                !/\.mdx?(\?|$)/.test(id) ||
                !code.includes(CANONICAL_REPO_NAME)
            ) {
                return null;
            }
            return { code: rewrite(code), map: null };
        },
    };
}
