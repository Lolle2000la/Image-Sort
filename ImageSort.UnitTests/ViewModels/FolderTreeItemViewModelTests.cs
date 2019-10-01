using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
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
                .Select(sub => path + sub) // make the (mock) subfolders absolute paths.
                .ToArray();

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(path)).Returns(resultingPaths);

            var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object) 
            { 
                Path = path
            };

            folderTreeItem.IsExpanded = true;

            var obtainedPaths = folderTreeItem.Children;

            fsMock.Verify(fs => fs.GetSubFolders(path));

            Assert.Equal(resultingPaths, obtainedPaths.Select(vm => vm.Path).ToArray());
        }

        [Fact(DisplayName = "Handles an tried access to an unauthorized file (UnauthorizedAccessException) gracefully.")]
        public void HandlesUnauthorizedAccessExceptionGracefully()
        {
            const string pathToUnauthorisedFolder = @"C:\UnauthorizedFolder";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(pathToUnauthorisedFolder)).Throws(new UnauthorizedAccessException());

            var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object)
            {
                Path = pathToUnauthorisedFolder
            };

            folderTreeItem.IsExpanded = true;

            fsMock.Verify(fs => fs.GetSubFolders(pathToUnauthorisedFolder));

            Assert.Null(folderTreeItem.Children);
        }

        [Fact(DisplayName = "Only allows expanding of the current folder (IsExpandable) if that is possible.")]
        public void OnlyAllowsExpansionWhenPossible()
        {
            const string pathToEmptyFolder = @"C:\EmptyFolder";
            const string pathToNonEmptyFolder = @"C:\NonEmptyFolder";
            const string pathToUnauthorisedFolder = @"C:\UnauthorizedFolder";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.IsFolderEmpty(pathToNonEmptyFolder)).Returns(false);
            fsMock.Setup(fs => fs.IsFolderEmpty(pathToEmptyFolder)).Returns(true);
            fsMock.Setup(fs => fs.IsFolderEmpty(pathToUnauthorisedFolder)).Throws(new UnauthorizedAccessException());

            var folderTreeItem = new FolderTreeItemViewModel(fsMock.Object, RxApp.MainThreadScheduler)
            {
                Path = pathToNonEmptyFolder
            };

            Assert.True(folderTreeItem.IsExpandable);
            fsMock.Verify(fs => fs.IsFolderEmpty(pathToNonEmptyFolder));

            folderTreeItem.Path = pathToEmptyFolder;
            Assert.False(folderTreeItem.IsExpandable);
            fsMock.Verify(fs => fs.IsFolderEmpty(pathToEmptyFolder));

            folderTreeItem.Path = pathToUnauthorisedFolder;
            Assert.False(folderTreeItem.IsExpandable);
            fsMock.Verify(fs => fs.IsFolderEmpty(pathToUnauthorisedFolder));
        }
    }
}
