using ImageSort.FileSystem;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Text;

namespace ImageSort.ViewModels
{
    public class FolderTreeItemViewModel : ReactiveObject
    {
        public string Path { get; }

        private readonly IFileSystem fileSystem;

        private bool _isExpanded = false;
        public bool IsExpanded
        {
            get => _isExpanded;
            set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
        }

        private readonly ObservableAsPropertyHelper<IEnumerable<FolderTreeItemViewModel>> _children;
        public IEnumerable<FolderTreeItemViewModel> Children => _children.Value;

        public FolderTreeItemViewModel(string path, IFileSystem fileSystem)
        {
            Path = path;
            this.fileSystem = fileSystem;

            _children = this
                .WhenAnyValue(x => x.IsExpanded)
                .Where(b => b)
                .Take(1)
                .CombineLatest(
                    Observable.Return(Path),
                    (_, p) => p)
                .Select(p => fileSystem
                    .GetSubFolders(p)
                    .Select(folder => new FolderTreeItemViewModel(folder, fileSystem)))
                .ToProperty(this, x => x.Children);
        }
    }
}
