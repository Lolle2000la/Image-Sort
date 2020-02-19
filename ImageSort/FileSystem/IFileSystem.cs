using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.FileSystem
{
    public interface IFileSystem
    {
        IEnumerable<string> GetSubFolders(string path);

        IEnumerable<string> GetFiles(string folder);

        bool IsFolderEmpty(string path);
    }
}
