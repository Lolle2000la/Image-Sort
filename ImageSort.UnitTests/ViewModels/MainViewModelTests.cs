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

        [Fact(DisplayName = "Does not open the currently selected folder if there is none or it is the current one.")]
        public async Task DoesNotOpenCurrentlySelectedFolderWhenItDoesntMakeSense()
        {
            Assert.True(await mainVM.OpenCurrentlySelectedFolder.CanExecute.FirstAsync()); // just to make sure the state is correct beforehand.

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
    }
}
