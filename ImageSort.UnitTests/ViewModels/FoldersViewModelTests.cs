using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using System.Linq;
using System.Reactive.Linq;
using System.Threading.Tasks;
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
            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = CreateMock(MockPath)
            };

            Assert.Equal(MockPath, foldersVM.CurrentFolder.Path);
        }

        [Fact(DisplayName = "Can prompt user to pin folders.")]
        public async Task CanPinFolders()
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

            await foldersVM.Pin.Execute();

            Assert.Contains(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }

        [Fact(DisplayName = "Handles user canceling gracefully.")]
        public async Task HandlesCancelingGracefully()
        {
            const string mockPathToPin = @"C:\SomeOtherPath\";

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = CreateMock(MockPath)
            };

            await foldersVM.Pin.Execute();

            Assert.DoesNotContain(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }

        [Fact(DisplayName = "Can pin the selected folder.")]
        public async Task CanPinSelected()
        {
            const string mockPathToPin = @"C:\SomeOtherPath\";

            var mockToPin = CreateMock(mockPathToPin);

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = CreateMock(MockPath)
            };

            foldersVM.Selected = mockToPin;

            await foldersVM.PinSelected.Execute();

            Assert.Contains(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }

        [Fact(DisplayName = "Can unpin the selected folder.")]
        public async Task CanUnpinSelected()
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

            await foldersVM.Pin.Execute();

            foldersVM.Selected = foldersVM.PinnedFolders
                .Where(pf => pf.Path == mockPathToPin)
                .First();

            await foldersVM.UnpinSelected.Execute();

            Assert.DoesNotContain(mockPathToPin, foldersVM.PinnedFolders
                .Select(f => f.Path));
        }

        [Fact(DisplayName = "Only unpins if pinned in the first place.")]
        public async Task OnlyUnpinsIfPinned()
        {
            var mockItem = CreateMock(MockPath);

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = mockItem,
                Selected = mockItem
            };

            Assert.False(await foldersVM.UnpinSelected.CanExecute.FirstAsync());
        }

        [Fact(DisplayName = "Concatenates the current folder and the pinned folders correctly.")]
        public async Task ConcatenatesFoldersCorrectly()
        {
            var currentFolder = CreateMock(MockPath);

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = currentFolder
            };

            var mockFolders = new[]
            {
                @"C:\SomeOtherPath1\",
                @"C:\SomeOtherPath2\",
                @"C:\SomeOtherPath3\"
            };

            foreach (var mockFolder in mockFolders)
            {
                foldersVM.SelectFolder.RegisterHandler(interaction =>
                {
                    interaction.SetOutput(mockFolder);
                });

                await foldersVM.Pin.Execute();
            }

            Assert.Equal(new[] { currentFolder.Path }.Concat(mockFolders),
                foldersVM.AllFoldersTracked.Select(f => f.Path));
        }

        [Fact(DisplayName = "Marks the current folder as such")]
        public async Task MarksCurrentFolder()
        {
            var currentFolder = CreateMock(MockPath);

            var foldersVM = new FoldersViewModel
            {
                CurrentFolder = currentFolder
            };

            Assert.True(foldersVM.CurrentFolder.IsCurrentFolder);
        }
    }
}
