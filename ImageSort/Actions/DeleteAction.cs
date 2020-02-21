using ImageSort.FileSystem;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace ImageSort.Actions
{
    /// <summary>
    /// Deletes a file. Only a file, not a directory.
    /// </summary>
    public class DeleteAction : IReversibleAction
    {
        private readonly string oldPath;
        private readonly IRecycleBin recycleBin;
        private IDisposable deletedFile;

        public DeleteAction(string path, IFileSystem fileSystem, IRecycleBin recycleBin)
        {
            if (path == null) throw new ArgumentNullException(nameof(path));
            if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
            if (recycleBin == null) throw new ArgumentNullException(nameof(recycleBin));
            if (!fileSystem.FileExists(path)) throw new FileNotFoundException(null, path);

            oldPath = path;
            this.recycleBin = recycleBin;
        }

        public void Act()
        {
            if (deletedFile == null) deletedFile = recycleBin.Send(oldPath);
        }

        public void Revert()
        {
            deletedFile?.Dispose();

            deletedFile = null;
        }
    }
}
