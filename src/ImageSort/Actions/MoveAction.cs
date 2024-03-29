﻿using System;
using System.IO;
using ImageSort.FileSystem;
using ImageSort.Localization;

namespace ImageSort.Actions;

public class MoveAction : IReversibleAction
{
    private readonly IFileSystem fileSystem;
    private readonly string oldDestination;
    private readonly string newDestination;
    private readonly Action<string, string> notifyAct;
    private readonly Action<string, string> notifyRevert;

    public MoveAction(string file, string toFolder, IFileSystem fileSystem,
        Action<string, string> notifyAct = null, Action<string, string> notifyRevert = null)
    {
        if (file == null) throw new ArgumentNullException(nameof(file));
        if (toFolder == null) throw new ArgumentNullException(nameof(toFolder));
        if (fileSystem == null) throw new ArgumentNullException(nameof(fileSystem));
        if (!fileSystem.FileExists(file)) throw new FileNotFoundException(null, file);
        if (!fileSystem.DirectoryExists(toFolder))
            throw new DirectoryNotFoundException(
                Text.DirectoryNotFoundExceptionMessage.Replace("{Directory}", toFolder,
                    StringComparison.OrdinalIgnoreCase));

        this.fileSystem = fileSystem;

        this.notifyAct = notifyAct;
        this.notifyRevert = notifyRevert;

        // ensure absolute paths, there are weird windows path limit bugs
        file = Path.GetFullPath(file);
        toFolder = Path.GetFullPath(toFolder);

        oldDestination = file;
        newDestination = Path.Combine(toFolder, Path.GetFileName(file));
    }

    public string DisplayName => Text.MoveActionMessage
        .Replace("{FileName}", Path.GetFileName(oldDestination), StringComparison.OrdinalIgnoreCase)
        .Replace("{Directory}", Path.GetDirectoryName(newDestination), StringComparison.OrdinalIgnoreCase);

    public void Act()
    {
        fileSystem.Move(oldDestination, newDestination);

        notifyAct?.Invoke(oldDestination, newDestination);
    }

    public void Revert()
    {
        fileSystem.Move(newDestination, oldDestination);

        notifyRevert?.Invoke(newDestination, oldDestination);
    }
}