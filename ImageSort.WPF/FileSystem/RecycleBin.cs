using ImageSort.FileSystem;
using System;
using System.IO;

namespace ImageSort.WPF.FileSystem
{
    class RecycleBin : IRecycleBin
    {
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

            throw new NotImplementedException();
        }
    }
}
