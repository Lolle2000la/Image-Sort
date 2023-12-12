using System.IO;
using ImageSort.Actions;
using ImageSort.FileSystem;
using NSubstitute;
using Xunit;

namespace ImageSort.UnitTests.Actions;

public class RenameActionTests
{
    [Fact(DisplayName = "Can rename files and undo")]
    public void CanRenameFilesAndUndo()
    {
        const string oldPath = @"C:\my-image.png";
        const string newFileName = "my-renamed-image";
        const string newPath = @"C:\my-renamed-image.png";

        var canAct = false;
        var canRevert = false;

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.FileExists(oldPath).Returns(true);
        fsMock.FileExists(newPath).Returns(false);

        var renameAction = new RenameAction(oldPath, newFileName, fsMock,
            (o, n) => canAct = true, (n, o) => canRevert = true);

        renameAction.Act();

        fsMock.Received().Move(oldPath, newPath);

        renameAction.Revert();

        fsMock.Received().Move(newPath, oldPath);

        Assert.True(canAct);
        Assert.True(canRevert);
    }

    [Fact(DisplayName = "Throws when the file doesn't exist or the renamed path is already used.")]
    public void ThrowsWhenFileDoesNotExistOrNewPathIsAlreadyUsed()
    {
        const string oldPath = @"C:\my-image.png";
        const string invalidOldPath = @"C:\invalid.gif";
        const string newFileName = "my-renamed-image";
        const string newPath = @"C:\my-renamed-image.png";
        const string alreadyExistingName = @"already-exists";
        const string alreadyExistingPath = @"C:\already-exists.png";

        var fsMock = Substitute.For<IFileSystem>();

        fsMock.FileExists(oldPath).Returns(true);
        fsMock.FileExists(newPath).Returns(false);
        fsMock.FileExists(invalidOldPath).Returns(false);
        fsMock.FileExists(alreadyExistingPath).Returns(true);

        Assert.Throws<FileNotFoundException>(() => new RenameAction(invalidOldPath, newFileName, fsMock));

        Assert.Throws<IOException>(() => new RenameAction(oldPath, alreadyExistingName, fsMock));

        fsMock.Received().FileExists(invalidOldPath);
        fsMock.Received().FileExists(alreadyExistingPath);
    }
}