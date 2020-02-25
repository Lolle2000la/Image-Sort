using DynamicData;
using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
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

        private readonly SourceList<FolderTreeItemViewModel> pinnedFolders;

        private readonly ReadOnlyObservableCollection<FolderTreeItemViewModel> _pinnedFolders;
        public ReadOnlyObservableCollection<FolderTreeItemViewModel> PinnedFolders => _pinnedFolders;

        private readonly ObservableAsPropertyHelper<IEnumerable<FolderTreeItemViewModel>> _allFoldersTracked;
        public IEnumerable<FolderTreeItemViewModel> AllFoldersTracked => _allFoldersTracked.Value;

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

        public FoldersViewModel(IFileSystem fileSystem = null, IScheduler backgroundScheduler = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();
            backgroundScheduler = backgroundScheduler ?? RxApp.TaskpoolScheduler;

            pinnedFolders = new SourceList<FolderTreeItemViewModel>();
            pinnedFolders.Connect()
                .ObserveOn(RxApp.MainThreadScheduler)
                .Bind(out _pinnedFolders)
                .Subscribe();

            _allFoldersTracked = this.WhenAnyValue(vm => vm.CurrentFolder)
                .CombineLatest(pinnedFolders.Connect(), (c, p) => (c, pinnedFolders.Items))
                .Select(folders => new[] { folders.c }.Concat(folders.Items))
                .ToProperty(this, vm => vm.AllFoldersTracked);

            Pin = ReactiveCommand.CreateFromTask(async () => 
            {
                try
                {
                    var folderToPin = await SelectFolder.Handle(Unit.Default);

                    pinnedFolders.Add(
                        new FolderTreeItemViewModel(fileSystem, backgroundScheduler)
                        {
                            Path = folderToPin
                        });
                }
                // an exception is ignored, because it only means that the 
                // user has canceled the dialog.
                catch (UnhandledInteractionException<Unit, string>) { }
            });

            var canPinSelectedExecute = this.WhenAnyValue(x => x.Selected, x => x.PinnedFolders.Count, (s, _) => s)
                .Select(s => s != null && !PinnedFolders.Contains(s));

            PinSelected = ReactiveCommand.Create(() =>
            {
                pinnedFolders.Add(Selected);
            }, canPinSelectedExecute);

            var canUnpinSelectedExecute = this.WhenAnyValue(vm => vm.Selected, x => x.PinnedFolders.Count, (s, _) => s)
                .Select(s => s != null && PinnedFolders.Contains(s));

            UnpinSelected = ReactiveCommand.Create(() =>
            {
                pinnedFolders.Remove(Selected);
            }, canUnpinSelectedExecute);

            // make many above queries work
            pinnedFolders.Add(null);
            pinnedFolders.RemoveAt(0);
        }

        ~FoldersViewModel()
        {
            pinnedFolders.Dispose();
        }
    }
}
