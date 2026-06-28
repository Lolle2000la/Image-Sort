using System;
using System.Collections.Generic;
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

public class FolderTreeItemViewModelTests
{
    [Fact(DisplayName = "Obtains the child folders of the current folder correctly")]
    public async Task ObtainsChildrenCorrectly()
    {
        var path = Path.GetFullPath("current_folder");

        var resultingPaths =
            new[]
            {
                "folder 1",
                "folder 2",
                "folder 3"
            }
            .Select(sub => Path.Combine(path, sub))
            .ToArray(); // make the (mock) subfolders absolute paths.

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(path).Returns(resultingPaths);

        var folderTreeItem = new FolderTreeItemViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
        {
            Path = path,
            IsVisible = true
        };

        while (folderTreeItem.Children.Count == 0) 
        {
            await Task.Delay(1);
        }

        fsMock.Received().GetSubFolders(path);

        Assert.Equal(resultingPaths.OrderBy(p => p), folderTreeItem.Children.Select(vm => vm.Path).OrderBy(p => p));
    }

    [Fact(DisplayName =
        "Handles an tried access to an unauthorized file (UnauthorizedAccessException) gracefully.")]
    public void HandlesUnauthorizedAccessExceptionGracefully()
    {
        var pathToUnauthorisedFolder = Path.GetFullPath("UnauthorizedFolder");

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(pathToUnauthorisedFolder).Returns(x => throw new UnauthorizedAccessException());

        var folderTreeItem = new FolderTreeItemViewModel(fsMock)
        {
            Path = pathToUnauthorisedFolder
        };
    }

    [Fact(DisplayName = "Can create folders and adds them to the children.")]
    public async Task CanCreateFolders()
    {
        var currentFolder = Path.GetFullPath("current_folder");
        var subfolders = new[]
        {
            "sub1", "sub2", "sub3"
        }.Select(s => Path.Combine(currentFolder, s));
        var addedFolder = Path.Combine(currentFolder, "new_sub");
        var result = new List<string>();
        result.AddRange(subfolders);
        result.Add(addedFolder);

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(currentFolder).Returns(subfolders);
        fsMock.CreateFolder(addedFolder);

        var folderTreeItem = new FolderTreeItemViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
        {
            Path = currentFolder,
            IsVisible = true
        };

        await folderTreeItem.CreateFolder.Execute("new_sub");
        // verify that no second folder is created when a folder already exists
        await folderTreeItem.CreateFolder.Execute("new_sub");

        await Task.Delay(1);

        fsMock.Received().CreateFolder(addedFolder);

        Assert.Equal(result.OrderBy(p => p), folderTreeItem.Children.Select(f => f.Path).OrderBy(p => p));
    }

    [Fact(DisplayName = "Do not load subfolders when not visible")]
    public void DoNotLoadSubfoldersWhenNotVisible()
    {
        var path = Path.GetFullPath("current_folder");

        var resultingPaths =
            new[]
            {
                "folder 1",
                "folder 2",
                "folder 3"
            }
            .Select(sub => Path.Combine(path, sub)); // make the (mock) subfolders absolute paths.

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.GetSubFolders(path).Returns(resultingPaths);

        var folderTreeItem = new FolderTreeItemViewModel(fsMock, backgroundScheduler: RxSchedulers.MainThreadScheduler)
        {
            Path = path,
            IsVisible = false
        };

        fsMock.DidNotReceive().GetSubFolders(path);

        Assert.Empty(folderTreeItem.Children.Select(vm => vm.Path).ToArray());
    }
}