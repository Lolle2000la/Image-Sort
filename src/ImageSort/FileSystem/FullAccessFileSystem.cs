using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace ImageSort.FileSystem
{
    public class FullAccessFileSystem : IFileSystem
    {
        public bool IsFolderEmpty(string path) => !Directory.EnumerateDirectories(path, "*", SearchOption.TopDirectoryOnly).Any();

        public IEnumerable<string> GetSubFolders(string path) => Directory.EnumerateDirectories(path, "*", SearchOption.TopDirectoryOnly);

        public IEnumerable<string> GetFiles(string folder) => Directory.EnumerateFiles(folder, "*", SearchOption.TopDirectoryOnly);
    }
}