using System.IO;
using ImageSort.Actions;
using ImageSort.FileSystem;
using NSubstitute;
using Xunit;

namespace ImageSort.UnitTests.Actions;

public class MoveActionTests
{
    [Fact(DisplayName = "File gets moved correctly and caller gets notified of change.")]
    public void FileGetsMovedCorrectly()
    {
        const string oldPath = @"C:\SomeFile.png";
        const string newFolder = @"C:\SomeOtherFolder\SomeOtherFolderAsWell\";
        const string newPath = newFolder + "SomeFile.png";

        var notifedOfAction = false;
        var notifiedOfReversion = false;

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.FileExists(oldPath).Returns(true);
        fsMock.DirectoryExists(newFolder).Returns(true);

        var moveAction = new MoveAction(oldPath, newFolder, fsMock,
            (f, t) => notifedOfAction = true,
            (f, t) => notifiedOfReversion = true);

        fsMock.Received().FileExists(oldPath);
        fsMock.Received().DirectoryExists(newFolder);

        moveAction.Act();

        fsMock.Received().Move(oldPath, newPath);

        moveAction.Revert();

        fsMock.Received().Move(newPath, oldPath);

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

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.DirectoryExists(existingDirectory).Returns(true);
        fsMock.FileExists(existingFile).Returns(true);
        fsMock.DirectoryExists(fakeDirectory).Returns(false);
        fsMock.FileExists(fakeFile).Returns(false);

        Assert.Throws<FileNotFoundException>(() => new MoveAction(fakeFile, existingDirectory, fsMock));

        Assert.Throws<DirectoryNotFoundException>(() => new MoveAction(existingFile, fakeDirectory, fsMock));
    }
}