using System;
using Xunit;
using ImageSort.ViewModels;
using System.Linq;
using ImageSort.FileSystem;
using Moq;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using System.Reactive.Linq;
using System.Reactive;

namespace ImageSort.UnitTests.ViewModels
{
    public class ImagesViewModelTests
    {
        [Fact(DisplayName = "Gets the files correctly, ignoring unsupported files.")]
        public void GetTheFilesCorrectly()
        {
            const string basePath = @"C:\";
            var allFiles = new[] { "image.png", "some.gif", "cat.mp4", "somethingwrong.exe" }
                .Select(f => basePath + f);

            var expectedFiles = allFiles.Where(f =>
                f.EndsWith(".png", StringComparison.OrdinalIgnoreCase)
                || f.EndsWith(".gif", StringComparison.OrdinalIgnoreCase));

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetFiles(basePath)).Returns(allFiles);

            var imagesVM = new ImagesViewModel(fsMock.Object) 
            { 
                CurrentFolder = basePath
            };

            fsMock.Verify(fs => fs.GetFiles(basePath));

            Assert.Equal(expectedFiles, imagesVM.Images);
        }

        [Fact(DisplayName = "Selected image is accessible by index and gives out the correct path.")]
        public void SelectedImageWorksCorrectly()
        {
            const string basePath = @"C:\";
            var allFiles = new[] { "image.png", "some.gif" }
                .Select(f => basePath + f);

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetFiles(basePath)).Returns(allFiles);

            var imagesVM = new ImagesViewModel(fsMock.Object)
            {
                CurrentFolder = basePath
            };

            imagesVM.SelectedIndex = 1;

            Assert.Equal(allFiles.ElementAt(1), imagesVM.SelectedImage);

            imagesVM.SelectedIndex = 0;

            Assert.Equal(allFiles.ElementAt(0), imagesVM.SelectedImage);
        }

        [Fact(DisplayName = "Can remove and add images externally")]
        public void CanRemoveAndAddImagesExternally()
        {
            const string basePath = @"C:\";
            var allFiles = new[] { "image.png", "some.gif" }
                .Select(f => basePath + f);

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetFiles(basePath)).Returns(allFiles);

            var imagesVM = new ImagesViewModel(fsMock.Object)
            {
                CurrentFolder = basePath
            };

            imagesVM.SelectedIndex = 0;

            Assert.Equal(allFiles, imagesVM.Images);

            imagesVM.RemoveImage(allFiles.ElementAt(1));

            Assert.Equal(allFiles.Where(p => p != allFiles.ElementAt(1)), imagesVM.Images);

            imagesVM.InsertImage(allFiles.ElementAt(1));

            Assert.Equal(allFiles, imagesVM.Images);
        }

        [Fact(DisplayName = "Search filter works")]
        public void SearchFilterWorks()
        {
            const string basePath = @"C:\";
            var allFiles = new[] { "image.png", "some.gif" }
                .Select(f => basePath + f);

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetFiles(basePath)).Returns(allFiles);

            var imagesVM = new ImagesViewModel(fsMock.Object)
            {
                CurrentFolder = basePath
            };

            imagesVM.SearchTerm = "image";

            Assert.DoesNotContain(allFiles.ElementAt(1), imagesVM.Images);

            Assert.Contains(allFiles.First(), imagesVM.Images);
        }

        [Fact(DisplayName = "Can rename images")]
        public async Task CanRenameImages()
        {
            const string basePath = @"C:\";
            const string oldFilePath = basePath + "image.png";
            const string newFileName = "other_image";
            var invalidFileNames = new[] { @"image\ima", "im/age", "imag\n", "imag\t" };
            var promptedFileName = newFileName;
            const string newFilePath = basePath + newFileName + ".png";
            var allFiles = new[] { oldFilePath };
            var allFilesResulting = new[] { newFilePath };

            var notifiesUserOfError = false;

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.GetFiles(basePath)).Returns(allFiles);

            fsMock.Setup(fs => fs.Move(oldFilePath, newFilePath)).Verifiable();

            fsMock.Setup(fs => fs.FileExists(oldFilePath)).Returns(true);

            var imagesVM = new ImagesViewModel(fsMock.Object)
            {
                CurrentFolder = basePath
            };

            imagesVM.RenameImage.Where(a => a != null)
                .Subscribe(a => a.Act());

            Assert.Equal(allFiles, imagesVM.Images);

            imagesVM.PromptForNewFileName.RegisterHandler(ic => ic.SetOutput(promptedFileName));
            imagesVM.NotifyUserOfError.RegisterHandler(ic =>
            {
                notifiesUserOfError = true;

                ic.SetOutput(Unit.Default);
            });

            await imagesVM.RenameImage.Execute();

            fsMock.Verify(fs => fs.Move(oldFilePath, newFilePath));

            await Task.Delay(1);

            Assert.Equal(allFilesResulting, imagesVM.Images);

            foreach (var invalidFileName in invalidFileNames)
            {
                promptedFileName = invalidFileName;

                await imagesVM.RenameImage.Execute();

                Assert.Equal(allFilesResulting, imagesVM.Images);
            }

            Assert.True(notifiesUserOfError);
        }
    }
}
