using System;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Threading.Tasks;
using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using ReactiveUI;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class MainViewModelTests
    {
        private readonly MainViewModel mainVM;

        public MainViewModelTests()
        {
            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(@"C:\")).Returns(new[] {@"C:\folder"});
            fsMock.Setup(fs => fs.GetSubFolders(It.IsAny<string>())).Returns(Enumerable.Empty<string>);

            fsMock.Setup(fs => fs.GetFiles(It.IsAny<string>()))
                .Returns(new[] {@"c:\img.png"}); // just so that no exception is thrown

            mainVM = new MainViewModel(fsMock.Object)
            {
                Images = new ImagesViewModel(fsMock.Object)
                {
                    CurrentFolder = @"C:\"
                },
                Folders = new FoldersViewModel(fsMock.Object)
                {
                    CurrentFolder = new FolderTreeItemViewModel(fsMock.Object) {Path = @"C:\"},
                    Selected = new FolderTreeItemViewModel(fsMock.Object) {Path = @"C:\folder"}
                }
            };
        }

        [Fact(DisplayName = "Can open the currently selected folder")]
        public async Task CanOpenCurrentlySelectedFolder()
        {
            await mainVM.OpenCurrentlySelectedFolder.Execute();

            Assert.Equal(@"C:\folder", mainVM.Folders.CurrentFolder.Path);
        }

        [Fact(DisplayName = "Does not open the currently selected folder if there is none or it is the current one.")]
        public async Task DoesNotOpenCurrentlySelectedFolderWhenItDoesntMakeSense()
        {
            Assert.True(await mainVM.OpenCurrentlySelectedFolder.CanExecute
                .FirstAsync()); // just to make sure the state is correct beforehand.

            mainVM.Folders.Selected = null;

            Assert.False(await mainVM.OpenCurrentlySelectedFolder.CanExecute.FirstAsync());

            mainVM.Folders.Selected = mainVM.Folders.CurrentFolder;

            Assert.False(await mainVM.OpenCurrentlySelectedFolder.CanExecute.FirstAsync());
        }

        [Fact(DisplayName = "Changes the path in the image view model to the current path when necessary")]
        public async Task ChangesThePathInImageViewModel()
        {
            await mainVM.OpenCurrentlySelectedFolder.Execute();

            Assert.Equal(@"C:\folder", mainVM.Images.CurrentFolder);
        }

        [Fact(DisplayName = "Properly selects the folder picked by the user when requested")]
        public async Task ProperlySelectsPickedFolder()
        {
            var requestsUserInput = false;

            mainVM.PickFolder.RegisterHandler(ic =>
            {
                requestsUserInput = true;

                ic.SetOutput(@"C:\SomeFolder");
            });

            await mainVM.OpenFolder.Execute();

            Assert.True(requestsUserInput);

            Assert.Equal(@"C:\SomeFolder", mainVM.Folders.CurrentFolder.Path);
        }

        [Fact(DisplayName =
            "Can move images to a folder and registers that action, removing the image from the images viewmodel in the process.")]
        public async Task CanMoveImages()
        {
            const string currentDirectory = @"C:\Some Folder With Pictures";
            const string image = currentDirectory + @"\some image.png";
            const string newDirectory = @"C:\Some other folder";
            const string moveDestination = newDirectory + @"\some image.png";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.DirectoryExists(currentDirectory)).Returns(true);
            fsMock.Setup(fs => fs.FileExists(image)).Returns(true);
            fsMock.Setup(fs => fs.DirectoryExists(newDirectory)).Returns(true);

            fsMock.Setup(fs => fs.GetFiles(currentDirectory)).Returns(new[] {image});

            fsMock.Setup(fs => fs.Move(image, moveDestination)).Verifiable();

            var otherMainVM = new MainViewModel(fsMock.Object, noParallel: true)
            {
                Actions = new ActionsViewModel(),
                Folders = new FoldersViewModel(fsMock.Object, RxApp.MainThreadScheduler, true)
                {
                    CurrentFolder = new FolderTreeItemViewModel(fsMock.Object, backgroundScheduler: RxApp.MainThreadScheduler)
                        {Path = currentDirectory}
                },
                Images = new ImagesViewModel(fsMock.Object)
            };

            otherMainVM.Images.SelectedIndex = 0;

            otherMainVM.Folders.SelectFolder.RegisterHandler(ic => ic.SetOutput(newDirectory));

            await otherMainVM.Folders.Pin.Execute();

            otherMainVM.Folders.Selected = otherMainVM.Folders.PinnedFolders.First();

            var actions = otherMainVM.Actions.Execute.Replay();
            actions.Connect();

            await otherMainVM.MoveImageToFolder.Execute();

            await actions.FirstAsync();

            Assert.Contains(Path.GetFileName(image), otherMainVM.Actions.LastDone, StringComparison.OrdinalIgnoreCase);

            Assert.Contains(Path.GetDirectoryName(moveDestination), otherMainVM.Actions.LastDone,
                StringComparison.OrdinalIgnoreCase);

            Assert.Empty(otherMainVM.Images.Images);

            fsMock.Verify(fs => fs.Move(image, moveDestination));
        }

        [Fact(DisplayName =
            "Can delete images and registers that action, removing the image from the images viewmodel in the process.")]
        public async Task CanDeleteImages()
        {
            const string currentDirectory = @"C:\Some Folder With Pictures";
            const string image = currentDirectory + @"\some image.png";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.DirectoryExists(currentDirectory)).Returns(true);
            fsMock.Setup(fs => fs.FileExists(image)).Returns(true);

            fsMock.Setup(fs => fs.GetFiles(currentDirectory)).Returns(new[] {image});

            var restorerMock = new Mock<IDisposable>();

            restorerMock.Setup(r => r.Dispose()).Verifiable();

            var rbMock = new Mock<IRecycleBin>();

            rbMock.Setup(rb => rb.Send(image, false)).Returns(restorerMock.Object);

            var otherMainVM = new MainViewModel(fsMock.Object, rbMock.Object)
            {
                Actions = new ActionsViewModel(),
                Folders = new FoldersViewModel(fsMock.Object, RxApp.MainThreadScheduler)
                {
                    CurrentFolder = new FolderTreeItemViewModel(fsMock.Object, backgroundScheduler: RxApp.MainThreadScheduler)
                        {Path = currentDirectory}
                },
                Images = new ImagesViewModel(fsMock.Object)
            };

            otherMainVM.Images.SelectedIndex = 0;

            await otherMainVM.DeleteImage.Execute();

            Assert.Contains(Path.GetFileName(image), otherMainVM.Actions.LastDone, StringComparison.OrdinalIgnoreCase);

            Assert.Empty(otherMainVM.Images.Images);

            await otherMainVM.Actions.Undo.Execute();

            restorerMock.Verify(r => r.Dispose());
        }
    }
}