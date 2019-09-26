using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;

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
    }
}
