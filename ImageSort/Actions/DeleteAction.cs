using ImageSort.FileSystem;
using System;
using System.IO;

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
        private readonly Action<string> notifyAct;
        private readonly Action<string> notifyRevert;

        public DeleteAction(string path, IFileSystem fileSystem, IRecycleBin recycleBin,
            Action<string> notifyAct = null, Action<string> notifyRevert = null)
        {
            if (path == null) throw new ArgumentNullException(nameof(path));
            if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
            if (recycleBin == null) throw new ArgumentNullException(nameof(recycleBin));
            if (!fileSystem.FileExists(path)) throw new FileNotFoundException(null, path);

            this.notifyAct = notifyAct;
            this.notifyRevert = notifyRevert;

            oldPath = path;
            this.recycleBin = recycleBin;
        }

        public string DisplayName => $"Delete {Path.GetFileName(oldPath)}";

        public void Act()
        {
            if (deletedFile == null) deletedFile = recycleBin.Send(oldPath);

            notifyAct?.Invoke(oldPath);
        }

        public void Revert()
        {
            deletedFile?.Dispose();

            deletedFile = null;

            notifyRevert?.Invoke(oldPath);
        }
    }
}
