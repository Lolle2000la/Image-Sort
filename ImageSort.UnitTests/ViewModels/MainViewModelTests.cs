using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class MainViewModelTests
    {
        private readonly MainViewModel mainVM;

        public MainViewModelTests()
        {
            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetSubFolders(@"C:\")).Returns(new[] { @"C:\folder" });

            mainVM = new MainViewModel()
            {
                Folders = new FoldersViewModel(fsMock.Object)
                {
                    CurrentFolder = new FolderTreeItemViewModel(fsMock.Object) { Path = @"C:\" },
                    Selected = new FolderTreeItemViewModel(fsMock.Object) { Path = @"C:\folder" }
                },
                Images = new ImagesViewModel()
            };
        }

        [Fact(DisplayName = "Can open the currently selected folder")]
        public async Task CanOpenCurrentlySelectedFolder()
        {
            await mainVM.OpenCurrentlySelectedFolder.Execute();

            Assert.Equal(@"C:\folder", mainVM.Folders.CurrentFolder.Path);
        }
    }
}
