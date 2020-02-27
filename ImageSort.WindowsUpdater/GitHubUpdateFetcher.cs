using Octokit;
using System;
using System.Collections.Generic;
using System.IO;
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
            throw new NotImplementedException();
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
