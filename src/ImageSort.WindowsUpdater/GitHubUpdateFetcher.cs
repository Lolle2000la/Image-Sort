using System;
using System.IO;
using System.Linq;
using System.Net;
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

    public async Task<(bool, Release)> TryGetLatestReleaseAsync(bool allowPrerelease = false)
    {
        var assembly = Assembly.GetAssembly(typeof(GitHubUpdateFetcher));
        var gitVersionInformationType = assembly.GetType("GitVersionInformation");
        var versionTag =
            (string) gitVersionInformationType?.GetFields().First(f => f.Name == "SemVer").GetValue(null);
        var version = SemVersion.Parse(versionTag, SemVersionStyles.Any);

        Release latestFitting;

        try
        {
            var releases = await client.Repository.Release.GetAll(RepoOwner, RepoName);

            latestFitting = releases
                .Where(release => IsV2Release(release.TagName))
                .Where(release =>
                {
                    var prereleaseCondition = allowPrerelease || !release.Prerelease;

                    var firstIndexOfV = release.TagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);

                    var releaseVersion = SemVersion.Parse(release.TagName.Substring(firstIndexOfV + 1), SemVersionStyles.Any);

                    var isNewVersion = version.ComparePrecedenceTo(releaseVersion) < 0;

                    return prereleaseCondition && isNewVersion;
                })
                .FirstOrDefault();
        }
        catch
        {
            latestFitting = null;
        }

        return (latestFitting != null, latestFitting);
    }

    public async Task<bool> HasStableV3ReleaseAsync()
    {
        try
        {
            var releases = await client.Repository.Release.GetAll(RepoOwner, RepoName);

            return releases
                .Where(release => IsV3Release(release.TagName))
                .Any(release => !release.Prerelease);
        }
        catch
        {
            return false;
        }
    }

    private static bool IsV2Release(string tagName)
    {
        var firstIndexOfV = tagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);
        if (firstIndexOfV < 0) return false;
        var versionPart = tagName.Substring(firstIndexOfV + 1);
        return versionPart.StartsWith("2.");
    }

    private static bool IsV3Release(string tagName)
    {
        var firstIndexOfV = tagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);
        if (firstIndexOfV < 0) return false;
        var versionPart = tagName.Substring(firstIndexOfV + 1);
        return versionPart.StartsWith("3.");
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
            return await httpClient.GetStreamAsync(asset.BrowserDownloadUrl);
        }
        catch (HttpRequestException)
        {
            return null;
        }
    }
}
