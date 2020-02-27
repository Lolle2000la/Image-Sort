using Octokit;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

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

        public bool TryGetLatestRelease(bool prerelease, out Release release)
        {
            throw new NotImplementedException();
        }

        public bool TryGetInstallerFromRelease(Release release, out ReleaseAsset installer)
        {
            throw new NotImplementedException();
        }

        public Stream GetStreamFromAsset(ReleaseAsset asset)
        {
            throw new NotImplementedException();
        }
    }
}
