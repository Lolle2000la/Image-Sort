using ReactiveUI;
using System.Collections.Generic;
using System.Reactive;

namespace ImageSort.ViewModels
{
    public class FoldersViewModel : ReactiveObject
    {
        private FolderTreeItemViewModel _currentFolder;
        public FolderTreeItemViewModel CurrentFolder
        {
            get => _currentFolder;
            set => this.RaiseAndSetIfChanged(ref _currentFolder, value);
        }

        private IEnumerable<FolderTreeItemViewModel> _pinnedFolders;
        public IEnumerable<FolderTreeItemViewModel> PinnedFolders
        {
            get => _pinnedFolders;
            set => this.RaiseAndSetIfChanged(ref _pinnedFolders, value);
        }

        private FolderTreeItemViewModel _selected;
        public FolderTreeItemViewModel Selected
        {
            get => _selected;
            set => this.RaiseAndSetIfChanged(ref _selected, value);
        }

        public ReactiveCommand<string, Unit> Pin { get; }
        public ReactiveCommand<Unit, Unit> PinSelected { get; }
        public ReactiveCommand<Unit, Unit> UnpinSelected { get; }
    }
}
