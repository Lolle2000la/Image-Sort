using ImageSort.FileSystem;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace ImageSort.Actions
{
    public class MoveAction : IReversibleAction
    {
        private readonly IFileSystem fileSystem;
        private readonly string oldDestination;
        private readonly string newDestination;

        public MoveAction(string file, string toFolder, IFileSystem fileSystem)
        {
            if (file == null) throw new ArgumentNullException(nameof(file));
            if (toFolder == null) throw new ArgumentNullException(nameof(toFolder));
            if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
            if (!fileSystem.FileExists(file)) throw new ArgumentException("The file to move should exist.", nameof(file));
            if (!fileSystem.DirectoryExists(toFolder)) throw new ArgumentException("The folder to move to should exist.", nameof(toFolder));

            this.fileSystem = fileSystem;

            // ensure absolute paths, there are weird windows path limit bugs
            file = Path.GetFullPath(file);
            toFolder = Path.GetFullPath(toFolder);

            oldDestination = file;
            newDestination = Path.Combine(toFolder, file);
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
