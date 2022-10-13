using System;
using System.IO;
using System.Reactive.Disposables;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using Shell32;

namespace ImageSort.WPF.FileSystem;

internal class RecycleBin : IRecycleBin
{
    private readonly Shell shell = new Shell();

    public IDisposable Send(string path, bool confirmationNeeded = false)
    {
        var success = false;

        if (confirmationNeeded)
            success = FileOperationApiWrapper.Send(path,
                FileOperationApiWrapper.FileOperationFlags.FOF_ALLOWUNDO
                | FileOperationApiWrapper.FileOperationFlags.FOF_WANTNUKEWARNING);
        else
            success = FileOperationApiWrapper.Send(path,
                FileOperationApiWrapper.FileOperationFlags.FOF_ALLOWUNDO
                | FileOperationApiWrapper.FileOperationFlags.FOF_NOCONFIRMATION
                | FileOperationApiWrapper.FileOperationFlags.FOF_WANTNUKEWARNING);

        if (!success) throw new IOException($"Could not delete {Path.GetFileName(path)}");

        return Disposable.Create(path, RestoreFileFromRecycleBin);
    }

    private void RestoreFileFromRecycleBin(string path)
    {
        var recycler = shell.NameSpace(10);

        foreach (FolderItem item in recycler.Items())
        {
            var fileName = recycler.GetDetailsOf(item, 0);

            if (string.IsNullOrEmpty(Path.GetExtension(fileName))) fileName += Path.GetExtension(item.Path);

            var filePath = recycler.GetDetailsOf(item, 1);

            if (path.PathEquals(Path.Combine(filePath, fileName)))
            {
                Restore(item);
                return;
            }
        }

        throw new FileNotFoundException(null, path);
    }

    private void Restore(FolderItem item)
    {
        var itemVerbs = item.Verbs();

        itemVerbs.Item(0).DoIt();
    }
}