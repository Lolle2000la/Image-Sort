using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.FileSystem
{
    public interface IFileSystem
    {
        IReadOnlyCollection<string> GetSubFolders(string path);

        IReadOnlyCollection<string> GetFiles(string folder);

        bool IsFolderEmpty(string path);
    }
}
