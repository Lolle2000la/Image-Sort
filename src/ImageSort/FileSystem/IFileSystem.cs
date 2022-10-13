using System.Collections.Generic;
using System.IO;

namespace ImageSort.FileSystem;

public interface IFileSystem
{
    IEnumerable<string> GetSubFolders(string path);

    IEnumerable<string> GetFiles(string folder);

    bool IsFolderEmpty(string path);

    bool FileExists(string path) => File.Exists(path);

    bool DirectoryExists(string path) => Directory.Exists(path);

    void Move(string source, string destination) => File.Move(source, destination);

    void CreateFolder(string path) => Directory.CreateDirectory(path);
}