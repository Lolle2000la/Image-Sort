using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ImageSort.Helpers;
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
        private readonly IFileSystem fileSystem;
        private readonly bool noParallel;

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

        public Interaction<Unit, string> PromptForName { get; }
            = new Interaction<Unit, string>();

        public ReactiveCommand<Unit, Unit> Pin { get; }
        public ReactiveCommand<Unit, Unit> PinSelected { get; }
        public ReactiveCommand<Unit, Unit> UnpinSelected { get; }

        public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderUp { get; }
        public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderDown { get; }

        public ReactiveCommand<Unit, Unit> CreateFolderUnderSelected { get; }

        public FoldersViewModel(IFileSystem fileSystem = null, IScheduler backgroundScheduler = null, bool noParallel = false)
        {
            this.fileSystem = fileSystem ??= Locator.Current.GetService<IFileSystem>();
            backgroundScheduler ??= RxApp.TaskpoolScheduler;
            this.noParallel = noParallel;

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

                    if (pinnedFolders.Items.Any(f => f.Path.PathEquals(folderToPin))) return;

                    pinnedFolders.Add(
                        new FolderTreeItemViewModel(fileSystem, noParallel: noParallel)
                        {
                            Path = folderToPin
                        });
                }
                // an exception is ignored, because it only means that the
                // user has canceled the dialog.
                catch (UnhandledInteractionException<Unit, string>) { }
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
                .Select(s => pinnedFolders.Items.Contains(s) && pinnedFolders.Items.IndexOf(s) > 0);

            MoveSelectedPinnedFolderUp = ReactiveCommand.Create(() =>
            {
                int pinnedIndex = pinnedFolders.Items.IndexOf(Selected);

                pinnedFolders.Move(pinnedIndex, pinnedIndex - 1);
            }, canMovePinnedFolderUp);

            var canMovePinnedFolderDown = this.WhenAnyValue(x => x.Selected)
                .Select(s => pinnedFolders.Items.Contains(s) && pinnedFolders.Items.IndexOf(s) < pinnedFolders.Count - 1);

            MoveSelectedPinnedFolderDown = ReactiveCommand.Create(() =>
            {
                int pinnedIndex = pinnedFolders.Items.IndexOf(Selected);

                pinnedFolders.Move(pinnedIndex, pinnedIndex + 1);
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

            FolderTreeItemViewModel oldFolder = null;

            this.WhenAnyValue(x => x.CurrentFolder)
                .Where(f => f != null)
                .Subscribe(f =>
                {
                    if (oldFolder != null) oldFolder.IsCurrentFolder = false;

                    f.IsCurrentFolder = true;

                    oldFolder = f;
                });
        }

        public void AddPinnedFoldersFromPaths(IEnumerable<string> paths)
        {
            pinnedFolders.AddRange(paths.Select(p =>
                {
                    if (!fileSystem.DirectoryExists(p)) return null;

                    try
                    {
                        return new FolderTreeItemViewModel(fileSystem, noParallel: noParallel) { Path = p };
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