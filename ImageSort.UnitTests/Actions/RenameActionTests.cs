using System;
using System.Collections.Generic;
using System.Text;
using ImageSort.Actions;
using ImageSort.FileSystem;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.Actions
{
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

            var fsMock = new Mock<IFileSystem>();

            fsMock.Setup(fs => fs.FileExists(oldPath)).Returns(true);
            fsMock.Setup(fs => fs.FileExists(newPath)).Returns(false);
            fsMock.Setup(fs => fs.Move(oldPath, newPath)).Verifiable();
            fsMock.Setup(fs => fs.Move(newPath, oldPath)).Verifiable();

            var renameAction = new RenameAction(oldPath, newFileName, fsMock.Object,
                (o, n) => canAct = true, (n, o) => canRevert = true);

            renameAction.Act();

            fsMock.Verify(fs => fs.Move(oldPath, newPath));

            renameAction.Revert();

            fsMock.Verify(fs => fs.Move(newPath, oldPath));

            Assert.True(canAct);
            Assert.True(canRevert);
        }
    }
}
