﻿using System;
using System.Collections.Generic;
using System.Text;
using Xunit;
using ImageSort.ViewModels;
using System.Linq;
using ImageSort.FileSystem;
using Moq;
using System.Collections.ObjectModel;

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

            fsMock.Setup(fs => fs.GetFiles(basePath))
                  .Returns(new ReadOnlyCollection<string>(allFiles.ToList()));

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

            fsMock.Setup(fs => fs.GetFiles(basePath))
                  .Returns(new ReadOnlyCollection<string>(allFiles.ToList()));

            var imagesVM = new ImagesViewModel(fsMock.Object)
            {
                CurrentFolder = basePath
            };

            imagesVM.SelectedIndex = 1;

            Assert.Equal(allFiles.ElementAt(1), imagesVM.SelectedImage);

            imagesVM.SelectedIndex = 0;

            Assert.Equal(allFiles.ElementAt(0), imagesVM.SelectedImage);
        }
    }
}
