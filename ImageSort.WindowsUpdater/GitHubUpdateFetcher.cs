using Octokit;
using Semver;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.WindowsUpdater
{
    public class GitHubUpdateFetcher
    {
        private readonly string currentVersion;
        private readonly GitHubClient client;

        public GitHubUpdateFetcher(string currentVersion, GitHubClient client)
        {
            this.currentVersion = currentVersion;
            this.client = client;
        }

        public async Task<(bool, Release)> TryGetLatestReleaseAsync(bool allowPrerelease=false)
        {
            var assembly = Assembly.GetAssembly(typeof(GitHubUpdateFetcher));
            var assemblyName = assembly.GetName().Name;
            var gitVersionInformationType = assembly.GetType(assemblyName + ".GitVersionInformation");
            dynamic gitVersionInformations = Activator.CreateInstance(gitVersionInformationType);
            var version = SemVersion.Parse((string) gitVersionInformations.SemVer);

            Release latestFitting;

            try
            {
                var releases = await client.Repository.Release.GetAll("Lolle2000la", "Image-Sort");

                latestFitting = releases
                    .FirstOrDefault(release => 
                    {
                        var prereleaseCondition = !allowPrerelease ? !release.Prerelease : true;

                        var releaseVersion = SemVersion.Parse(release.TagName);

                        var isNewVersion = version.CompareTo(releaseVersion) < 0;

                        return prereleaseCondition && isNewVersion;
                    });
            }
            catch (ApiException)
            {
                latestFitting = null;
            }

            return (latestFitting != null, latestFitting);
        }

        public async Task<(bool, ReleaseAsset)> TryGetInstallerFromReleaseAsync(Release release)
        {
            throw new NotImplementedException();
        }

        public async Task<Stream> GetStreamFromAssetAsync(ReleaseAsset asset)
        {
            throw new NotImplementedException();
        }
    }
}
