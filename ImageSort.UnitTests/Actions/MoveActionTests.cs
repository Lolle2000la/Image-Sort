using System;
using System.Collections.Generic;
using System.Text;
using ImageSort.Actions;
using ImageSort.FileSystem;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.Actions
{
    public class MoveActionTests
    {
        [Fact(DisplayName = "File gets moved correctly.")]
        public void FileGetsMovedCorrectly()
        {
            const string oldPath = @"C:\SomeFile.png";
            const string newFolder = @"C:\SomeOtherFolder\SomeOtherFolderAsWell\";
            const string newPath = newFolder + "SomeFile.png";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.Move(oldPath, newPath)).Verifiable();
            fsMock.Setup(fs => fs.Move(newPath, oldPath)).Verifiable();
            fsMock.Setup(fs => fs.FileExists(oldPath)).Returns(true);
            fsMock.Setup(fs => fs.DirectoryExists(newFolder)).Returns(true);

            var moveAction = new MoveAction(oldPath, newFolder, fsMock.Object);

            fsMock.Verify(fs => fs.FileExists(oldPath));
            fsMock.Verify(fs => fs.DirectoryExists(newFolder));

            moveAction.Act();

            fsMock.Verify(fs => fs.Move(oldPath, newPath));

            moveAction.Revert();

            fsMock.Verify(fs => fs.Move(newPath, oldPath));
        }

        [Fact(DisplayName = "Handles file or directory not existing correctly.")]
        public void HandlesFileOrDirectoryNotExisting()
        {
            const string existingDirectory = @"C:\SomeRealDirectory";
            const string existingFile = @"C:\SomeRealFile.tif";
            const string fakeDirectory = @"C:\DirectoryThatDoesntExist";
            const string fakeFile = @"C:\SomeFakeFile.gif";

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.DirectoryExists(existingDirectory)).Returns(true);
            fsMock.Setup(fs => fs.FileExists(existingFile)).Returns(true);
            fsMock.Setup(fs => fs.DirectoryExists(fakeDirectory)).Returns(false);
            fsMock.Setup(fs => fs.FileExists(fakeFile)).Returns(false);

            Assert.Throws<ArgumentException>("file", () => new MoveAction(fakeFile, existingDirectory, fsMock.Object));

            Assert.Throws<ArgumentException>("toFolder", () => new MoveAction(existingFile, fakeDirectory, fsMock.Object));
        }
    }
}
