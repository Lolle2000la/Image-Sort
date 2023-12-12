using System;
using System.IO;
using ImageSort.Actions;
using ImageSort.FileSystem;
using NSubstitute;
using Xunit;

namespace ImageSort.UnitTests.Actions;

public class DeleteActionTests
{
    [Fact(DisplayName = "Deletes and Restores the file correctly.")]
    public void DeletesAndRestoresTheFileCorrectly()
    {
        const string fileToDelete = @"C:\Some File.png";

        var fsMock = Substitute.For<IFileSystem>();
        var recycleBinMock = Substitute.For<IRecycleBin>();
        var fileRestorerMock = Substitute.For<IDisposable>();

        fsMock.FileExists(fileToDelete).Returns(true);

        recycleBinMock.Send(fileToDelete, false).Returns(fileRestorerMock);

        var deleteAction = new DeleteAction(fileToDelete, fsMock, recycleBinMock);

        fsMock.Received().FileExists(fileToDelete);

        deleteAction.Act();

        recycleBinMock.Received().Send(fileToDelete, false);

        deleteAction.Revert();

        fileRestorerMock.Received().Dispose();
    }

    [Fact(DisplayName = "Throws when the file to delete does not exist")]
    public void ThrowsWhenTheFileDoesNotExist()
    {
        const string fileThatDoesntExist = @"C:\Fictional File.fake";

        var fsMock = Substitute.For<IFileSystem>();
        var recycleBinMock = Substitute.For<IRecycleBin>();

        fsMock.FileExists(fileThatDoesntExist).Returns(false);

        Assert.Throws<FileNotFoundException>(() =>
            new DeleteAction(fileThatDoesntExist, fsMock, recycleBinMock));

        fsMock.Received().FileExists(fileThatDoesntExist);
    }
}