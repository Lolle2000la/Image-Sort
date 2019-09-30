using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text;

namespace ImageSort.FileSystem
{
    public class FullAccessFileSystem : IFileSystem
    {
        public bool IsFolderEmpty(string path) => !Directory.EnumerateDirectories(path).Any();

        public IReadOnlyCollection<string> GetSubFolders(string path) => new ReadOnlyCollection<string>(Directory.GetDirectories(path));
    }
}
