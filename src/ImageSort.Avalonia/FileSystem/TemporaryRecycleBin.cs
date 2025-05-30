using System;
using System.IO;
using ImageSort.FileSystem;
using Splat;

namespace ImageSort.Avalonia.FileSystem
{
    public class TemporaryRecycleBin : IRecycleBin
    {
        private readonly IFileSystem _fileSystem;
        private const string RecycleBinFolderName = "_ImageSort_RecycleBin_";

        public TemporaryRecycleBin(IFileSystem fileSystem = null)
        {
            _fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();
        }

        public IDisposable Send(string path, bool confirmationNeeded = false)
        {
            if (string.IsNullOrEmpty(path)) throw new ArgumentNullException(nameof(path));
            if (!_fileSystem.FileExists(path)) throw new FileNotFoundException("File not found.", path);

            var fileInfo = new FileInfo(path);
            var parentDirectory = fileInfo.DirectoryName;
            if (parentDirectory == null) throw new InvalidOperationException("Cannot determine parent directory.");

            var recycleBinDirectory = Path.Combine(parentDirectory, RecycleBinFolderName);
            _fileSystem.CreateFolder(recycleBinDirectory);

            var fileName = fileInfo.Name;
            var newPathInRecycleBin = Path.Combine(recycleBinDirectory, fileName);

            // Handle potential name conflicts in the recycle bin
            int count = 1;
            string uniqueFileName = fileName;
            while (_fileSystem.FileExists(newPathInRecycleBin))
            {
                uniqueFileName = $"{Path.GetFileNameWithoutExtension(fileName)}_{count++}{Path.GetExtension(fileName)}";
                newPathInRecycleBin = Path.Combine(recycleBinDirectory, uniqueFileName);
            }

            _fileSystem.Move(path, newPathInRecycleBin);

            return new Restorer(_fileSystem, newPathInRecycleBin, path);
        }

        private class Restorer : IDisposable
        {
            private readonly IFileSystem _fs;
            private readonly string _recycledPath;
            private readonly string _originalPath;
            private bool _disposed = false;

            public Restorer(IFileSystem fs, string recycledPath, string originalPath)
            {
                _fs = fs;
                _recycledPath = recycledPath;
                _originalPath = originalPath;
            }

            public void Dispose()
            {
                if (_disposed) return;

                try
                {
                    if (_fs.FileExists(_recycledPath))
                    {
                        // Ensure original directory exists
                        var originalDir = Path.GetDirectoryName(_originalPath);
                        if(originalDir != null && !_fs.DirectoryExists(originalDir))
                        {
                            _fs.CreateFolder(originalDir);
                        }
                        
                        // Check if original file name is now taken, if so, find a new name
                        string pathToRestore = _originalPath;
                        int count = 1;
                        while(_fs.FileExists(pathToRestore))
                        {
                            pathToRestore = $"{Path.Combine(Path.GetDirectoryName(_originalPath), Path.GetFileNameWithoutExtension(_originalPath))}_{count++}{Path.GetExtension(_originalPath)}";
                        }

                        _fs.Move(_recycledPath, pathToRestore);
                    }
                }
                catch (Exception ex)
                {
                    // Log or handle restoration failure
                    // For now, we'll rethrow as FileRestorationNotPossibleException for consistency,
                    // though the original interface implies this is for when the recycle bin itself fails.
                    throw new FileRestorationNotPossibleException($"Failed to restore file from temporary recycle bin: {ex.Message}", ex);
                }
                finally
                {
                    _disposed = true;
                }
            }
        }
    }
}
