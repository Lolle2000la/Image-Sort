using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Reflection;
using System.Threading.Tasks;
using Octokit;
using Semver;

namespace ImageSort.WindowsUpdater;

public class GitHubUpdateFetcher
{
    private readonly GitHubClient client;
    private const string RepoOwner = "Lolle2000la";
    private const string RepoName = "Image-Sort";

    public GitHubUpdateFetcher(GitHubClient client)
    {
        this.client = client;
    }

    public async Task<(bool updateFound, Release release, bool hasStableV3)> TryGetLatestReleaseAsync(bool allowPrerelease = false)
    {
        var assembly = Assembly.GetAssembly(typeof(GitHubUpdateFetcher));
        var gitVersionInformationType = assembly.GetType("GitVersionInformation");
        var versionTag =
            (string) gitVersionInformationType?.GetFields().First(f => f.Name == "SemVer").GetValue(null);
        var version = SemVersion.Parse(versionTag, SemVersionStyles.Any);

        Release latestFitting = null;

        try
        {
            var releases = await client.Repository.Release.GetAll(RepoOwner, RepoName);

            latestFitting = releases
                .Where(release => IsV2Release(release.TagName))
                .Where(release =>
                {
                    var prereleaseCondition = allowPrerelease || !release.Prerelease;

                    var firstIndexOfV = release.TagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);
                    var versionString = release.TagName.Substring(firstIndexOfV + 1);

                    if (!SemVersion.TryParse(versionString, SemVersionStyles.Any, out var releaseVersion))
                        return false;

                    var isNewVersion = version.ComparePrecedenceTo(releaseVersion) < 0;

                    return prereleaseCondition && isNewVersion;
                })
                .FirstOrDefault();

            var hasStableV3 = releases
                .Where(release => IsV3Release(release.TagName))
                .Any(release => !release.Prerelease);

            return (latestFitting != null, latestFitting, hasStableV3);
        }
        catch
        {
            return (false, null, false);
        }
    }

    private static bool IsV2Release(string tagName)
    {
        if (string.IsNullOrEmpty(tagName)) return false;
        var firstIndexOfV = tagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);
        if (firstIndexOfV < 0) return false;
        var versionPart = tagName.Substring(firstIndexOfV + 1);
        return versionPart.StartsWith("2.", StringComparison.Ordinal);
    }

    private static bool IsV3Release(string tagName)
    {
        if (string.IsNullOrEmpty(tagName)) return false;
        var firstIndexOfV = tagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);
        if (firstIndexOfV < 0) return false;
        var versionPart = tagName.Substring(firstIndexOfV + 1);
        return versionPart.StartsWith("3.", StringComparison.Ordinal);
    }

    public bool TryGetInstallerFromRelease(Release release, out ReleaseAsset installer)
    {
        if (release == null) throw new ArgumentNullException(nameof(release));

        var is64bit = Environment.Is64BitProcess;

        installer = release.Assets
            .FirstOrDefault(asset => asset.Name
                .Equals($"ImageSort.{(is64bit ? "x64" : "x86")}.msi", StringComparison.OrdinalIgnoreCase));

        return installer != null;
    }

    public async Task<Stream> GetStreamFromAssetAsync(ReleaseAsset asset)
    {
        var handler = new HttpClientHandler
        {
            AllowAutoRedirect = true
        };

        using var httpClient = new HttpClient(handler);

        httpClient.DefaultRequestHeaders.Add("User-Agent", "Image-Sort");

        try
        {
            var responseBytes = await httpClient.GetByteArrayAsync(asset.BrowserDownloadUrl);
            return new MemoryStream(responseBytes);
        }
        catch (HttpRequestException)
        {
            return null;
        }
    }
}
