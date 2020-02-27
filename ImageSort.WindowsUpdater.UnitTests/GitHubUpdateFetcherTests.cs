using Moq;
using Octokit;
using System;
using System.Collections.Generic;
using System.Text;
using Xunit;

namespace ImageSort.WindowsUpdater.UnitTests
{
    public class GitHubUpdateFetcherTests
    {
        [Fact(DisplayName = "Can fetch the latest updates and turn them into a stream for consumption.")]
        public void CanFetchLatestUpdates()
        {
            var ghubClient = new Mock<GitHubClient>();
        }
    }
}
