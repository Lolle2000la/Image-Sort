using System;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Threading.Tasks;
using ImageSort.FileSystem;
using ImageSort.ViewModels;
using NSubstitute;
using ReactiveUI;
using Xunit;

namespace ImageSort.UnitTests.ViewModels;

public class MainViewModelTests
{
    private readonly MainViewModel mainVM;

    public MainViewModelTests()
    {
        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(Path.GetFullPath("root")).Returns(new[] { Path.GetFullPath(Path.Combine("root", "folder")) });
        fsMock.GetSubFolders(Arg.Any<string>()).Returns(Enumerable.Empty<string>());

        fsMock.GetFiles(Arg.Any<string>())
            .Returns(new[] { Path.GetFullPath("img.png") }); // just so that no exception is thrown

        mainVM = new MainViewModel(fsMock)
        {
            Images = new ImagesViewModel(fsMock)
            {
                CurrentFolder = Path.GetFullPath("root")
            },
            Folders = new FoldersViewModel(fsMock)
            {
                CurrentFolder = new FolderTreeItemViewModel(fsMock) { Path = Path.GetFullPath("root") },
                Selected = new FolderTreeItemViewModel(fsMock) { Path = Path.GetFullPath(Path.Combine("root", "folder")) }
            }
        };
    }

    [Fact(DisplayName = "Can open the currently selected folder")]
    public async Task CanOpenCurrentlySelectedFolder()
    {
        await mainVM.OpenCurrentlySelectedFolder.Execute();

        Assert.Equal(Path.GetFullPath(Path.Combine("root", "folder")), mainVM.Folders.CurrentFolder.Path);
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

        Assert.Equal(Path.GetFullPath(Path.Combine("root", "folder")), mainVM.Images.CurrentFolder);
    }

    [Fact(DisplayName = "Properly selects the folder picked by the user when requested")]
    public async Task ProperlySelectsPickedFolder()
    {
        var requestsUserInput = false;

        mainVM.PickFolder.RegisterHandler(ic =>
        {
            requestsUserInput = true;

            ic.SetOutput(Path.GetFullPath("SomeFolder"));
        });

        await mainVM.OpenFolder.Execute();

        Assert.True(requestsUserInput);

        Assert.Equal(Path.GetFullPath("SomeFolder"), mainVM.Folders.CurrentFolder.Path);
    }

    [Fact(DisplayName =
        "Can move images to a folder and registers that action, removing the image from the images viewmodel in the process.")]
    public async Task CanMoveImages()
    {
        var currentDirectory = Path.GetFullPath("pictures");
        var image = Path.Combine(currentDirectory, "some image.png");
        var newDirectory = Path.GetFullPath("other");
        var moveDestination = Path.Combine(newDirectory, "some image.png");

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.DirectoryExists(currentDirectory).Returns(true);
        fsMock.FileExists(image).Returns(true);
        fsMock.DirectoryExists(newDirectory).Returns(true);

        fsMock.GetFiles(currentDirectory).Returns(new[] {image});

        var otherMainVM = new MainViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
        {
            Actions = new ActionsViewModel(),
            Folders = new FoldersViewModel(fsMock, RxSchedulers.MainThreadScheduler)
            {
                CurrentFolder = new FolderTreeItemViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
                    {Path = currentDirectory}
            },
            Images = new ImagesViewModel(fsMock)
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

        fsMock.Received().Move(image, moveDestination);
    }

    [Fact(DisplayName = "Can delete images and registers that action, removing the image from the images viewmodel in the process.")]
    public async Task CanDeleteImages()
    {
        var currentDirectory = Path.GetFullPath("pictures_delete");
        var image = Path.Combine(currentDirectory, "some image.png");

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.DirectoryExists(currentDirectory).Returns(true);
        fsMock.FileExists(image).Returns(true);

        fsMock.GetFiles(currentDirectory).Returns(new[] { image });

        var restorerMock = Substitute.For<IDisposable>();

        var rbMock = Substitute.For<IRecycleBin>();

        rbMock.Send(image, false).Returns(restorerMock);

        var otherMainVM = new MainViewModel(fsMock, rbMock, RxSchedulers.MainThreadScheduler)
        {
            Actions = new ActionsViewModel(),
            Folders = new FoldersViewModel(fsMock, RxSchedulers.MainThreadScheduler)
            {
                CurrentFolder = new FolderTreeItemViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
                { Path = currentDirectory }
            },
            Images = new ImagesViewModel(fsMock)
        };

        otherMainVM.Images.SelectedIndex = 0;

        await otherMainVM.DeleteImage.Execute();

        await Task.Delay(1); // Give OAPH time to update

        Assert.Contains(Path.GetFileName(image), otherMainVM.Actions.LastDone, StringComparison.OrdinalIgnoreCase);

        Assert.Empty(otherMainVM.Images.Images);

        await otherMainVM.Actions.Undo.Execute();

        restorerMock.Received().Dispose();
    }
}