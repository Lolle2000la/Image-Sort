using System;
using System.IO;
using ImageSort.FileSystem;
using ImageSort.Localization;

namespace ImageSort.Actions;

public class RenameAction : IReversibleAction
{
    private readonly IFileSystem fileSystem;
    private readonly string newPath;
    private readonly string oldPath;
    private readonly Action<string, string> notifyAct;
    private readonly Action<string, string> notifyRevert;

    public RenameAction(string path, string newName, IFileSystem fileSystem,
        Action<string, string> notifyAct = null, Action<string, string> notifyRevert = null)
    {
        if (path == null) throw new ArgumentNullException(nameof(path));
        if (newName == null) throw new ArgumentNullException(nameof(newName));
        if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
        if (!fileSystem.FileExists(path)) throw new FileNotFoundException(null, path);

        oldPath = path = Path.GetFullPath(path);
        newPath = Path.Combine(Path.GetDirectoryName(path), newName + Path.GetExtension(path));

        if (fileSystem.FileExists(newPath))
            throw new IOException(
                Text.FileAlreadyExistsExceptionMessage.Replace("{FileName}", newName,
                    StringComparison.OrdinalIgnoreCase));

        this.fileSystem = fileSystem;

        this.notifyAct = notifyAct;
        this.notifyRevert = notifyRevert;
    }

    public string DisplayName => Text.RenameActionMessage
        .Replace("{OldFileName}", Path.GetFileName(oldPath), StringComparison.OrdinalIgnoreCase)
        .Replace("{NewFileName}", Path.GetFileName(newPath), StringComparison.OrdinalIgnoreCase);

    public void Act()
    {
        fileSystem.Move(oldPath, newPath);

        notifyAct?.Invoke(oldPath, newPath);
    }

    public void Revert()
    {
        fileSystem.Move(newPath, oldPath);

        notifyRevert?.Invoke(newPath, oldPath);
    }
}