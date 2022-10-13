using System;
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
        var version = SemVersion.Parse(versionTag);

        Release latestFitting;

        try
        {
            var releases = await client.Repository.Release.GetAll("Lolle2000la", "Image-Sort");

            latestFitting = releases
                .FirstOrDefault(release =>
                {
                    var prereleaseCondition = allowPrerelease || !release.Prerelease;

                    var firstIndexOfV = release.TagName.IndexOf('v', StringComparison.OrdinalIgnoreCase);

                    var releaseVersion = SemVersion.Parse(release.TagName.Substring(firstIndexOfV + 1));

                    var isNewVersion = version.CompareByPrecedence(releaseVersion) < 0;

                    return prereleaseCondition && isNewVersion;
                });
        }
        catch
        {
            latestFitting = null;
        }

        return (latestFitting != null, latestFitting);
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
        using var httpClient = new HttpClient();

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