using ImageSort.FileSystem;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;

namespace ImageSort.ViewModels
{
    public class FolderTreeItemViewModel : ReactiveObject
    {
        public string Path { get; }

        private readonly IFileSystem fileSystem;

        public IEnumerable<FolderTreeItemViewModel> Children => fileSystem
            .GetSubFolders(Path)
            .Select(folder => new FolderTreeItemViewModel(folder, fileSystem));

        public FolderTreeItemViewModel(string path, IFileSystem fileSystem)
        {
            Path = path;
            this.fileSystem = fileSystem;
        }
    }
}
