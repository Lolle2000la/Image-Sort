using System;
using System.IO;
using ImageSort.Actions;
using ImageSort.FileSystem;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.Actions
{
    public class DeleteActionTests
    {
        [Fact(DisplayName = "Deletes and Restores the file correctly.")]
        public void DeletesAndRestoresTheFileCorrectly()
        {
            const string fileToDelete = @"C:\Some File.png";

            var fsMock = new Mock<IFileSystem>();
            var recycleBinMock = new Mock<IRecycleBin>();
            var fileRestorerMock = new Mock<IDisposable>();

            fsMock.Setup(fs => fs.FileExists(fileToDelete)).Returns(true).Verifiable();

            fileRestorerMock.Setup(fr => fr.Dispose()).Verifiable();

            recycleBinMock.Setup(recycleBin => recycleBin.Send(fileToDelete, false)).Returns(fileRestorerMock.Object)
                .Verifiable();

            var deleteAction = new DeleteAction(fileToDelete, fsMock.Object, recycleBinMock.Object);

            fsMock.Verify(fs => fs.FileExists(fileToDelete));

            deleteAction.Act();

            recycleBinMock.Verify(recycleBin => recycleBin.Send(fileToDelete, false));

            deleteAction.Revert();

            fileRestorerMock.Verify(fr => fr.Dispose());
        }

        [Fact(DisplayName = "Throws when the file to delete does not exist")]
        public void ThrowsWhenTheFileDoesNotExist()
        {
            const string fileThatDoesntExist = @"C:\Fictional File.fake";

            var fsMock = new Mock<IFileSystem>();
            var recycleBinMock = new Mock<IRecycleBin>();
            var fileRestorerMock = new Mock<IDisposable>();

            fsMock.Setup(fs => fs.FileExists(fileThatDoesntExist)).Returns(false).Verifiable();

            Assert.Throws<FileNotFoundException>(() =>
                new DeleteAction(fileThatDoesntExist, fsMock.Object, recycleBinMock.Object));

            fsMock.Verify(fs => fs.FileExists(fileThatDoesntExist));
        }
    }
}