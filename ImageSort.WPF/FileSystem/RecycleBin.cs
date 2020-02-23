using ImageSort.FileSystem;
using Shell32;
using System;
using System.IO;
using System.Reactive.Disposables;

namespace ImageSort.WPF.FileSystem
{
    class RecycleBin : IRecycleBin
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
            Folder recycler = shell.NameSpace(10);

            foreach(FolderItem item in recycler.Items())
            {
                var fileName = recycler.GetDetailsOf(item, 0);

                if (string.IsNullOrEmpty(Path.GetExtension(fileName))) fileName += Path.GetExtension(item.Path);

                var filePath = recycler.GetDetailsOf(item, 1);

                if (path == Path.Combine(filePath, fileName))
                {
                    DoVerb(item, "ESTORE");
                    return;
                }
            }

            throw new FileNotFoundException(null, path);
        }

        private void DoVerb(FolderItem item, string verb)
        {
            var itemVerbs = item.Verbs();

            itemVerbs.Item(0).DoIt();
        }
    }
}
