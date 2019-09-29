using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Text;

namespace ImageSort.FileSystem
{
    public class FullAccessFileSystem : IFileSystem
    {
        public IReadOnlyCollection<string> GetSubFolders(string path) => new ReadOnlyCollection<string>(Directory.GetDirectories(path));
    }
}
