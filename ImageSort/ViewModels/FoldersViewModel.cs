using DynamicData;
using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Reactive;
using System.Reactive.Linq;

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

        private readonly ReadOnlyObservableCollection<FolderTreeItemViewModel> _pinnedFolders;
        public ReadOnlyObservableCollection<FolderTreeItemViewModel> PinnedFolders => _pinnedFolders;

        private FolderTreeItemViewModel _selected;
        public FolderTreeItemViewModel Selected
        {
            get => _selected;
            set => this.RaiseAndSetIfChanged(ref _selected, value);
        }

        /// <summary>
        /// Should prompt the user to select a folder.
        /// </summary>
        public Interaction<Unit, string> SelectFolder { get; } 
            = new Interaction<Unit, string>();

        public ReactiveCommand<Unit, Unit> Pin { get; }
        public ReactiveCommand<Unit, Unit> PinSelected { get; }
        public ReactiveCommand<Unit, Unit> UnpinSelected { get; }

        public FoldersViewModel()
        {
            var pinnedFolders = new SourceList<FolderTreeItemViewModel>();
            pinnedFolders.Connect()
                .Bind(out _pinnedFolders)
                .Subscribe();

            Pin = ReactiveCommand.Create(() => 
            {
                SelectFolder.Handle(Unit.Default)
                    .Subscribe(folder => pinnedFolders.Add(
                        new FolderTreeItemViewModel(
                            Locator.Current.GetService<IFileSystem>())
                        { 
                            Path = folder
                        }));
            });
        }
    }
}
