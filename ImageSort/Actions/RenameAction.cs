using ImageSort.FileSystem;
using ImageSort.Localization;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace ImageSort.Actions
{
    public class RenameAction : IReversibleAction
    {
        private readonly IFileSystem fileSystem;
        private readonly Action<string, string> notifyAct;
        private readonly Action<string, string> notifyRevert;
        private readonly string oldPath;
        private readonly string newPath;

        public string DisplayName => Text.RenameActionMessage
            .Replace("{OldFileName}", Path.GetFileName(oldPath), StringComparison.OrdinalIgnoreCase)
            .Replace("{NewFileName}", Path.GetFileName(newPath), StringComparison.OrdinalIgnoreCase);

        public RenameAction(string path, string newName, IFileSystem fileSystem,
            Action<string, string> notifyAct = null, Action<string, string> notifyRevert = null)
        {
            if (path == null) throw new ArgumentNullException(nameof(path));
            if (newName == null) throw new ArgumentNullException(nameof(newName));
            if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
            if (!fileSystem.FileExists(path)) throw new FileNotFoundException(null, path);

            oldPath = path = Path.GetFullPath(path);
            newPath = Path.Combine(Path.GetDirectoryName(path), newName + Path.GetExtension(path));

            if (fileSystem.FileExists(newPath)) throw new IOException($"The file \"{newName}\" already exists.");

            this.fileSystem = fileSystem;

            this.notifyAct = notifyAct;
            this.notifyRevert = notifyRevert;
        }

        public void Act()
        {
            notifyAct?.Invoke(oldPath, newPath);

            fileSystem.Move(oldPath, newPath);
        }

        public void Revert()
        {
            notifyRevert?.Invoke(newPath, oldPath);

            fileSystem.Move(newPath, oldPath);
        }
    }
}
