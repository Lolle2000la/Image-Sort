using System;
using System.IO;
using ImageSort.FileSystem;
using ImageSort.Localization;

namespace ImageSort.Actions
{
    /// <summary>
    ///     Deletes a file. Only a file, not a directory.
    /// </summary>
    public class DeleteAction : IReversibleAction
    {
        private readonly Action<string> notifyAct;
        private readonly Action<string> notifyRevert;
        private readonly string oldPath;
        private readonly IRecycleBin recycleBin;
        private IDisposable deletedFile;

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

        public string DisplayName => Text.DeleteActionMessage
            .Replace("{FileName}", Path.GetFileName(oldPath), StringComparison.OrdinalIgnoreCase);

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