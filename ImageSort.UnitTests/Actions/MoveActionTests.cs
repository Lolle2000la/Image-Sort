using System.IO;
using ImageSort.Actions;
using ImageSort.FileSystem;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.Actions
{
    public class MoveActionTests
    {
        [Fact(DisplayName = "File gets moved correctly and caller gets notified of change.")]
        public void FileGetsMovedCorrectly()
        {
            const string oldPath = @"C:\SomeFile.png";
            const string newFolder = @"C:\SomeOtherFolder\SomeOtherFolderAsWell\";
            const string newPath = newFolder + "SomeFile.png";

            bool notifedOfAction = false;
            bool notifiedOfReversion = false;

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.Move(oldPath, newPath)).Verifiable();
            fsMock.Setup(fs => fs.Move(newPath, oldPath)).Verifiable();
            fsMock.Setup(fs => fs.FileExists(oldPath)).Returns(true);
            fsMock.Setup(fs => fs.DirectoryExists(newFolder)).Returns(true);

            var moveAction = new MoveAction(oldPath, newFolder, fsMock.Object, 
                (f,t) => notifedOfAction = true,
                (f,t) => notifiedOfReversion = true);

            fsMock.Verify(fs => fs.FileExists(oldPath));
            fsMock.Verify(fs => fs.DirectoryExists(newFolder));

            moveAction.Act();

            fsMock.Verify(fs => fs.Move(oldPath, newPath));

            moveAction.Revert();

            fsMock.Verify(fs => fs.Move(newPath, oldPath));

            Assert.True(notifedOfAction, "The caller should be notified when an action acts.");
            Assert.True(notifiedOfReversion, "The caller should be notified when an action is reverted.");
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

            Assert.Throws<FileNotFoundException>(() => new MoveAction(fakeFile, existingDirectory, fsMock.Object));

            Assert.Throws<DirectoryNotFoundException>(() => new MoveAction(existingFile, fakeDirectory, fsMock.Object));
        }
    }
}
