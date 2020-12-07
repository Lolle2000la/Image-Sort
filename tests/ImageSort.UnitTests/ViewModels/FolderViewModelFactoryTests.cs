using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Microsoft.Reactive.Testing;
using Moq;
using ReactiveUI;
using System;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class FolderViewModelFactoryTests
    {
        [Fact(DisplayName = "Factory produces FolderViewModel with correct value assignment/constructor arguments")]
        public void FactoryProducesCorrectFolderViewModel()
        {
            const string path = @"C:\current folder";

            var resultingPaths = Array.Empty<string>();

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(path)).Returns(resultingPaths).Verifiable();

            TestScheduler testScheduler = new();

            FolderViewModelFactory factory = new(fsMock.Object, () => null, testScheduler);

            var folder = factory.GetFor(path);
            folder.IsVisible = true;

            folder.WhenAnyValue(x => x.Path)
                .Subscribe(testScheduler.CreateObserver<object>());

            Assert.Equal(path, folder.Path);

            Assert.Throws<MockException>(() => fsMock.Verify(fs => fs.GetSubFolders(path)));

            testScheduler.AdvanceBy(TimeSpan.FromSeconds(1).Ticks);

            fsMock.Verify(fs => fs.GetSubFolders(path));

            testScheduler.Stop();
        }
    }
}
