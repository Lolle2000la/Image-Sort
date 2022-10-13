using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Threading.Tasks;
using Castle.Core.Internal;
using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Microsoft.Reactive.Testing;
using Moq;
using ReactiveUI;
using Xunit;

namespace ImageSort.UnitTests.ViewModels;

public class FolderTreeItemViewModelTests
{
    [Fact(DisplayName = "Obtains the child folders of the current folder correctly")]
    public void ObtainsChildrenCorrectly()
    {
        const string path = @"C:\current folder";

        var resultingPaths =
            new[]
                {
                    @"\folder 1",
                    @"\folder 2",
                    @"\folder 3"
                }
                .Select(sub => path + sub); // make the (mock) subfolders absolute paths.

        var fsMock = new Mock<IFileSystem>();

        fsMock.Setup(fs => fs.GetSubFolders(path)).Returns(resultingPaths).Verifiable();

        var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object, backgroundScheduler: RxApp.MainThreadScheduler)
        {
            Path = path,
            IsVisible = true
        };

        fsMock.Verify(fs => fs.GetSubFolders(path));

        while (folderTreeItem.Children.Count == 0) {}

        Assert.Equal(resultingPaths, folderTreeItem.Children.Select(vm => vm.Path).ToArray());
    }

    [Fact(DisplayName =
        "Handles an tried access to an unauthorized file (UnauthorizedAccessException) gracefully.")]
    public void HandlesUnauthorizedAccessExceptionGracefully()
    {
        const string pathToUnauthorisedFolder = @"C:\UnauthorizedFolder";

        var fsMock = new Mock<IFileSystem>();

        fsMock.Setup(fs => fs.GetSubFolders(pathToUnauthorisedFolder)).Throws(new UnauthorizedAccessException());

        var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object)
        {
            Path = pathToUnauthorisedFolder
        };
    }

    [Fact(DisplayName = "Can create folders and adds them to the children.")]
    public async Task CanCreateFolders()
    {
        const string currentFolder = @"C:\current_folder";
        var subfolders = new[]
        {
            "sub1", "sub2", "sub3"
        }.Select(s => Path.Combine(currentFolder, s));
        const string addedFolder = currentFolder + @"\new_ sub";
        var result = new List<string>();
        result.AddRange(subfolders);
        result.Add(addedFolder);

        var fsMock = new Mock<IFileSystem>();

        fsMock.Setup(fs => fs.GetSubFolders(currentFolder)).Returns(subfolders);
        fsMock.Setup(fs => fs.CreateFolder(addedFolder)).Verifiable();

        var testScheduler = new TestScheduler();

        testScheduler.Start();

        var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object, backgroundScheduler: RxApp.MainThreadScheduler)
        {
            Path = currentFolder,
            IsVisible = true
        };

        testScheduler.AdvanceBy(1);

        await folderTreeItem.CreateFolder.Execute(addedFolder);
        // verify that no second folder is created when a folder already exists
        await folderTreeItem.CreateFolder.Execute(addedFolder);

        testScheduler.AdvanceBy(1);

        fsMock.Verify(fs => fs.CreateFolder(addedFolder));

        Assert.Equal(result.OrderBy(p => p), folderTreeItem.Children.Select(f => f.Path).OrderBy(p => p));
    }

    [Fact(DisplayName = "Do not load subfolders when not visible")]
    public async Task DoNotLoadSubfoldersWhenNotVisible()
    {
        const string path = @"C:\current folder";

        var resultingPaths =
            new[]
                {
                    @"\folder 1",
                    @"\folder 2",
                    @"\folder 3"
                }
                .Select(sub => path + sub); // make the (mock) subfolders absolute paths.

        var fsMock = new Mock<IFileSystem>();

        fsMock.Setup(fs => fs.GetSubFolders(path)).Returns(resultingPaths).Verifiable();

        var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object, backgroundScheduler: RxApp.MainThreadScheduler)
        {
            Path = path,
            IsVisible = false
        };

        fsMock.Verify(fs => fs.GetSubFolders(path), Times.Never());

        Assert.Empty(folderTreeItem.Children.Select(vm => vm.Path).ToArray());
    }
}