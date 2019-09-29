using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Threading.Tasks;
using System.Text;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class FoldersViewModelTests
    {
        private const string MockPath = @"C:\SomePath\";

        private FolderTreeItemViewModel CreateMock(string path)
        {
            var fsMock = new Mock<IFileSystem>();

            return new FolderTreeItemViewModel(fsMock.Object)
            {
                Path = path
            };
        }

        [Fact(DisplayName = "The currently loaded folder can be changed.")]
        public void CurrentFolderCanBeChanged()
        {
            var foldersVM = new FoldersViewModel();

            foldersVM.CurrentFolder = CreateMock(MockPath);

            Assert.Equal(MockPath, foldersVM.CurrentFolder.Path);
        }

        [Fact(DisplayName = "Can prompt user to pin folders.")]
        public void CanPinFolders()
        {
            const string mockPathToPin = @"C:\SomeOtherPath\";

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = CreateMock(MockPath)
            };

            foldersVM.SelectFolder.RegisterHandler(interaction =>
            {
                interaction.SetOutput(mockPathToPin);
            });

            foldersVM.Pin.Execute().ToTask().Wait();

            Assert.NotNull(foldersVM.PinnedFolders);

            Assert.Contains(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }

        [Fact(DisplayName = "Handles user canceling gracefully.")]
        public void HandlesCancelingGracefully()
        {
            const string mockPathToPin = @"C:\SomeOtherPath\";

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = CreateMock(MockPath)
            };

            foldersVM.Pin.Execute();

            Assert.DoesNotContain(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }
    }
}
