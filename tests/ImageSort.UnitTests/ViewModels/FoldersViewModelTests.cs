using System;
using System.Linq;
using System.Reactive.Linq;
using System.Threading.Tasks;
using ImageSort.FileSystem;
using ImageSort.ViewModels;
using NSubstitute;
using Xunit;

namespace ImageSort.UnitTests.ViewModels;
public class FoldersViewModelTests
{
    private const string MockPath = @"C:\SomePath\";
    private readonly IFileSystem fileSystemMock;

    public FoldersViewModelTests()
    {
        var fsMock = Substitute.For<IFileSystem>();
        fsMock.GetSubFolders(Arg.Any<string>()).Returns(Array.Empty<string>());

        fileSystemMock = fsMock;
    }

    private FolderTreeItemViewModel CreateMock(string path)
    {
        return new FolderTreeItemViewModel(fileSystemMock)
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

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(Arg.Any<string>()).Returns(Enumerable.Empty<string>());

        var foldersVM = new FoldersViewModel(fsMock)
        {
            CurrentFolder = CreateMock(MockPath)
        };

        foldersVM.SelectFolder.RegisterHandler(interaction => { interaction.SetOutput(mockPathToPin); });

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

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(Arg.Any<string>()).Returns(Enumerable.Empty<string>());

        var foldersVM = new FoldersViewModel(fsMock)
        {
            CurrentFolder = CreateMock(MockPath)
        };

        foldersVM.SelectFolder.RegisterHandler(interaction => { interaction.SetOutput(mockPathToPin); });

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

    [Fact(DisplayName = "Marks the current folder as such")]
    public void MarksCurrentFolder()
    {
        var currentFolder = CreateMock(MockPath);

        var foldersVM = new FoldersViewModel
        {
            CurrentFolder = currentFolder
        };

        Assert.True(foldersVM.CurrentFolder.IsCurrentFolder);
    }

    [Fact(DisplayName = "Moves the selected pinned folder up correctly if possible")]
    public async Task MovesSelectedFolderUp()
    {
        var foldersVM = new FoldersViewModel(fileSystemMock)
        {
            CurrentFolder = CreateMock(MockPath)
        };

        var pinnedFolders = new[] { @"C:\folder 1", @"C:\folder 2", @"C:\folder 3" };

        foreach (var pinnedFolder in pinnedFolders)
        {
            foldersVM.SelectFolder.RegisterHandler(ic => ic.SetOutput(pinnedFolder));

            await foldersVM.Pin.Execute();
        }

        foldersVM.Selected = foldersVM.PinnedFolders.ElementAt(0);

        Assert.False(await foldersVM.MoveSelectedPinnedFolderUp.CanExecute.FirstAsync());

        var selected = foldersVM.PinnedFolders.ElementAt(1);
        foldersVM.Selected = selected;

        Assert.True(await foldersVM.MoveSelectedPinnedFolderUp.CanExecute.FirstAsync());

        await foldersVM.MoveSelectedPinnedFolderUp.Execute();

        Assert.Equal(selected, foldersVM.PinnedFolders.ElementAt(0));
    }

    [Fact(DisplayName = "Moves the selected pinned folder down correctly if possible")]
    public async Task MovesSelectedFolderDown()
    {
        var foldersVM = new FoldersViewModel(fileSystemMock)
        {
            CurrentFolder = CreateMock(MockPath)
        };

        var pinnedFolders = new[] { @"C:\folder 1", @"C:\folder 2", @"C:\folder 3" };

        foreach (var pinnedFolder in pinnedFolders)
        {
            foldersVM.SelectFolder.RegisterHandler(ic => ic.SetOutput(pinnedFolder));

            await foldersVM.Pin.Execute();
        }

        foldersVM.Selected = foldersVM.PinnedFolders.ElementAt(2);

        Assert.False(await foldersVM.MoveSelectedPinnedFolderDown.CanExecute.FirstAsync());

        var selected = foldersVM.PinnedFolders.ElementAt(1);
        foldersVM.Selected = selected;

        Assert.True(await foldersVM.MoveSelectedPinnedFolderDown.CanExecute.FirstAsync());

        await foldersVM.MoveSelectedPinnedFolderDown.Execute();

        Assert.Equal(selected, foldersVM.PinnedFolders.ElementAt(2));
    }
}