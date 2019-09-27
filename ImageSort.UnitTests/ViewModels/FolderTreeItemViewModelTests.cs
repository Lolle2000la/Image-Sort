using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
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

            var resultingPaths = new ReadOnlyCollection<string>(
                new[]
                {
                    @"\folder 1",
                    @"\folder 2",
                    @"\folder 3"
                }
                .Select(sub => path + sub) // make the (mock) subfolders absolute paths.
                .ToArray());

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(path)).Returns(resultingPaths);

            var folderTreeItem = new FolderTreeItemViewModel(path, fsMock.Object);

            folderTreeItem.IsExpanded = true;

            var obtainedPaths = folderTreeItem.Children;

            fsMock.Verify(fs => fs.GetSubFolders(path));

            Assert.Equal(resultingPaths, obtainedPaths.Select(vm => vm.Path));
        }
    }
}
