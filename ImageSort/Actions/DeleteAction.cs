using ImageSort.FileSystem;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace ImageSort.Actions
{
    public class DeleteAction : IReversibleAction
    {
        private readonly string oldPath;
        private readonly IFileSystem fileSystem;
        private readonly IRecycleBin recycleBin;

        public DeleteAction(string path, IFileSystem fileSystem, IRecycleBin recycleBin)
        {
            if (path == null) throw new ArgumentNullException(nameof(path));
            if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
            if (recycleBin == null) throw new ArgumentNullException(nameof(recycleBin));
            if (!fileSystem.FileExists(path)) throw new FileNotFoundException(null, path);

            oldPath = path;
            this.fileSystem = fileSystem;
            this.recycleBin = recycleBin;
        }

        public void Act()
        {
            throw new NotImplementedException();
        }

        public void Revert()
        {
            throw new NotImplementedException();
        }
    }
}
