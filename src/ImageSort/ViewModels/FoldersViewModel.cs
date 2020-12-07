using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive;
using System.Reactive.Linq;

namespace ImageSort.ViewModels
{
    public class FoldersViewModel : ReactiveObject
    {
        private readonly FolderViewModelFactory folderFactory;
        private readonly IFileSystem fileSystem;

        private FolderViewModel _currentFolder;

        public FolderViewModel CurrentFolder
        {
            get => _currentFolder;
            set => this.RaiseAndSetIfChanged(ref _currentFolder, value);
        }

        private readonly SourceList<FolderViewModel> pinnedFolders;

        private readonly ReadOnlyObservableCollection<FolderViewModel> _pinnedFolders;
        public ReadOnlyObservableCollection<FolderViewModel> PinnedFolders => _pinnedFolders;

        private readonly ObservableAsPropertyHelper<IEnumerable<FolderViewModel>> _allFoldersTracked;
        public IEnumerable<FolderViewModel> AllFoldersTracked => _allFoldersTracked.Value;

        private FolderViewModel _selected;

        public FolderViewModel Selected
        {
            get => _selected;
            set => this.RaiseAndSetIfChanged(ref _selected, value);
        }

        /// <summary>
        /// Should prompt the user to select a folder.
        /// </summary>
        public Interaction<Unit, string> SelectFolder { get; }
            = new Interaction<Unit, string>();

        public Interaction<Unit, string> PromptForName { get; }
            = new Interaction<Unit, string>();

        public ReactiveCommand<Unit, Unit> Pin { get; }
        public ReactiveCommand<Unit, Unit> PinSelected { get; }
        public ReactiveCommand<Unit, Unit> UnpinSelected { get; }
        
        public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderUp { get; }
        public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderDown { get; }

        public ReactiveCommand<Unit, Unit> CreateFolderUnderSelected { get; }

        public FoldersViewModel(FolderViewModelFactory folderFactory, IFileSystem fileSystem)
        {
            this.folderFactory = folderFactory;
            this.fileSystem = fileSystem;

            pinnedFolders = new SourceList<FolderViewModel>();
            pinnedFolders.Connect()
                .ObserveOn(RxApp.MainThreadScheduler)
                .Bind(out _pinnedFolders)
                .Subscribe();

            _allFoldersTracked = this.WhenAnyValue(vm => vm.CurrentFolder)
                .CombineLatest(pinnedFolders.Connect(), (c, p) => (c, pinnedFolders.Items))
                .Select(folders => new[] {folders.c}.Concat(folders.Items))
                .ToProperty(this, vm => vm.AllFoldersTracked);

            Pin = ReactiveCommand.CreateFromTask(async () =>
            {
                try
                {
                    var folderToPin = await SelectFolder.Handle(Unit.Default);

                    if (pinnedFolders.Items.Any(f => f.Path.PathEquals(folderToPin))) return;

                    pinnedFolders.Add(folderFactory.GetFor(folderToPin));
                }
                catch (UnhandledInteractionException<Unit, string>)
                {
                    // an exception is ignored, because it only means that the
                    // user has canceled the dialog.                
                }
            });

            var canPinSelectedExecute = this.WhenAnyValue(x => x.Selected, x => x.PinnedFolders.Count, (s, _) => s)
                .Select(s => s != null && !PinnedFolders.Where(f => f != null)
                    .Select(f => f.Path)
                    .Contains(s.Path));

            PinSelected = ReactiveCommand.Create(() =>
            {
                pinnedFolders.Add(Selected);
            }, canPinSelectedExecute);

            var canUnpinSelectedExecute = this.WhenAnyValue(vm => vm.Selected, x => x.PinnedFolders.Count, (s, _) => s)
                .Select(s => s != null && PinnedFolders.Where(f => f != null)
                    .Select(f => f.Path)
                    .Contains(s.Path));

            UnpinSelected = ReactiveCommand.Create(() =>
            {
                var pinned = pinnedFolders.Items.FirstOrDefault(f => f.Path.PathEquals(Selected.Path));

                if (pinned != null) pinnedFolders.Remove(pinned);
            }, canUnpinSelectedExecute);

            var canMovePinnedFolderUp = this.WhenAnyValue(x => x.Selected)
                .CombineLatest(PinnedFolders.ToObservableChangeSet(), (f, _) => f)
                .Select(s => pinnedFolders.Items.Contains(s) && pinnedFolders.Items.IndexOf(s) > 0);

            MoveSelectedPinnedFolderUp = ReactiveCommand.Create(() =>
            {
                var pinnedIndex = pinnedFolders.Items.IndexOf(Selected);

                if (pinnedIndex > 0) pinnedFolders.Move(pinnedIndex, pinnedIndex - 1);
            }, canMovePinnedFolderUp);

            var canMovePinnedFolderDown = this.WhenAnyValue(x => x.Selected)
                .CombineLatest(PinnedFolders.ToObservableChangeSet(), (f, _) => f)
                .Select(s =>
                    pinnedFolders.Items.Contains(s) && pinnedFolders.Items.IndexOf(s) < pinnedFolders.Count - 1);

            MoveSelectedPinnedFolderDown = ReactiveCommand.Create(() =>
            {
                var pinnedIndex = pinnedFolders.Items.IndexOf(Selected);

                if (pinnedIndex < pinnedFolders.Count - 1) pinnedFolders.Move(pinnedIndex, pinnedIndex + 1);
            }, canMovePinnedFolderDown);

            // make many above queries work
            pinnedFolders.Add(null);
            pinnedFolders.RemoveAt(0);

            CreateFolderUnderSelected = ReactiveCommand.CreateFromTask(async () =>
            {
                var name = await PromptForName.Handle(Unit.Default);

                if (string.IsNullOrEmpty(name)) return Unit.Default;

                return await Selected.CreateFolder.Execute(name);
            });

            FolderViewModel oldFolder = null;

            this.WhenAnyValue(x => x.CurrentFolder)
                .Where(f => f != null)
                .Subscribe(f =>
                {
                    if (oldFolder != null) oldFolder.IsCurrentFolder = false;

                    f.IsCurrentFolder = true;

                    oldFolder = f;
                });
        }

        [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1031:Do not catch general exception types", Justification = "There are many sorts of reasons why a folder cannot be accessed, but all of them can be ignored.")]
        public void AddPinnedFoldersFromPaths(IEnumerable<string> paths)
        {
            pinnedFolders.AddRange(paths.Select(p =>
                {
                    if (!fileSystem.DirectoryExists(p)) return null;

                    try
                    {
                        return folderFactory.GetFor(p);
                    }
                    catch { return null; }
                }).Where(f => f != null));
        }

        ~FoldersViewModel()
        {
            pinnedFolders.Dispose();
        }
    }
}