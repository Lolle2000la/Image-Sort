#nullable enable
using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO; 
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
// Removed System.Windows.Input; as ICommand is part of ReactiveUI

namespace ImageSort.ViewModels;

public class FoldersViewModel : ReactiveObject
{
    private readonly IFileSystem _fileSystem; // Renamed for consistency
    private readonly IScheduler _backgroundScheduler; // Renamed for consistency
    private readonly Func<FileSystemWatcher> _folderWatcherFactory; // Added for FolderTreeItemViewModel

    private FolderTreeItemViewModel? _currentFolder; // Made nullable

    public FolderTreeItemViewModel? CurrentFolder // Made nullable
    {
        get => _currentFolder;
        set => this.RaiseAndSetIfChanged(ref _currentFolder, value);
    }

    private readonly SourceList<FolderTreeItemViewModel> _pinnedFoldersSourceList; // Renamed

    private readonly ReadOnlyObservableCollection<FolderTreeItemViewModel> _pinnedFoldersBinding; // Renamed
    public ReadOnlyObservableCollection<FolderTreeItemViewModel> PinnedFolders => _pinnedFoldersBinding;

    private readonly ObservableAsPropertyHelper<IReadOnlyList<FolderTreeItemViewModel>> _displayedFolderItemsOaph; // Renamed
    public IReadOnlyList<FolderTreeItemViewModel> DisplayedFolderItems => _displayedFolderItemsOaph.Value;

    private readonly ObservableAsPropertyHelper<IEnumerable<FolderTreeItemViewModel>> _allFoldersTrackedOaph; // Renamed
    public IEnumerable<FolderTreeItemViewModel> AllFoldersTracked => _allFoldersTrackedOaph.Value;

    private FolderTreeItemViewModel? _selected; // Made nullable

    public FolderTreeItemViewModel? Selected // Made nullable
    {
        get => _selected;
        set => this.RaiseAndSetIfChanged(ref _selected, value);
    }

    public Interaction<Unit, string?> SelectFolder { get; } // Return type nullable
        = new Interaction<Unit, string?>();

    public Interaction<Unit, string?> PromptForName { get; } // Return type nullable
        = new Interaction<Unit, string?>();

    public ReactiveCommand<Unit, Unit> Pin { get; }
    public ReactiveCommand<Unit, Unit> PinSelected { get; }
    public ReactiveCommand<Unit, Unit> UnpinSelected { get; }

    public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderUp { get; }
    public ReactiveCommand<Unit, Unit> MoveSelectedPinnedFolderDown { get; }

    public ReactiveCommand<Unit, Unit> CreateFolderUnderSelected { get; }

    public ReactiveCommand<Unit, Unit> SelectNextFolder { get; } 
    public ReactiveCommand<Unit, Unit> SelectPreviousFolder { get; } 
    public ReactiveCommand<Unit, Unit> ExpandSelected { get; } 
    public ReactiveCommand<Unit, Unit> CollapseSelectedOrGoToParent { get; }
    public ReactiveCommand<Unit, Unit> SetSelectedFolderAsCurrentImplicitly { get; }
    public ReactiveCommand<Unit, Unit> PinCurrentFolder { get; }
    
    public ReactiveCommand<Unit, Unit> GoToParentFolderCommand { get; }
    public ReactiveCommand<Unit, Unit> OpenSelectedFolderCommand { get; }


    public FoldersViewModel(IFileSystem? fileSystem = null, IScheduler? backgroundScheduler = null, Func<FileSystemWatcher>? folderWatcherFactory = null)
    {
        _fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>() ?? throw new ArgumentNullException(nameof(fileSystem));
        _backgroundScheduler = backgroundScheduler ?? RxApp.TaskpoolScheduler;
        _folderWatcherFactory = folderWatcherFactory ?? (() => Locator.Current.GetService<FileSystemWatcher>()!);


        _pinnedFoldersSourceList = new SourceList<FolderTreeItemViewModel>();
        _pinnedFoldersSourceList.Connect()
            .ObserveOn(RxApp.MainThreadScheduler)
            .Bind(out _pinnedFoldersBinding)
            .Subscribe();

        var pinnedFoldersObservable = _pinnedFoldersSourceList.Connect().ToCollection().StartWith(new List<FolderTreeItemViewModel>());

        _displayedFolderItemsOaph = this.WhenAnyValue(x => x.CurrentFolder)
            .CombineLatest(pinnedFoldersObservable,
                           (current, pinned) => (Current: current, Pinned: pinned))
            // StartWith the current values to ensure an initial emission
            .StartWith((Current: this.CurrentFolder, Pinned: _pinnedFoldersSourceList.Items.ToList() as IReadOnlyCollection<FolderTreeItemViewModel> ?? new List<FolderTreeItemViewModel>()))
            .Select(data =>
            {
                var list = new List<FolderTreeItemViewModel>();
                if (data.Current != null)
                {
                    list.Add(data.Current);
                }
                // Ensure pinned folders are distinct and not the current folder if current is already added
                list.AddRange(data.Pinned.Where(pf => pf != null && (data.Current == null || !pf.Path.PathEquals(data.Current.Path))).DistinctBy(pf => pf.Path));
                return (IReadOnlyList<FolderTreeItemViewModel>)list.AsReadOnly(); // Return as ReadOnly for safety
            })
            .ToProperty(this, vm => vm.DisplayedFolderItems, initialValue: new List<FolderTreeItemViewModel>().AsReadOnly(), scheduler: RxApp.MainThreadScheduler);


        _allFoldersTrackedOaph = this.WhenAnyValue(vm => vm.CurrentFolder)
            .CombineLatest(pinnedFoldersObservable, (c, pItems) => (Current: c, PinnedItems: pItems))
            .Select(folders =>
            {
                var list = new List<FolderTreeItemViewModel>();
                if (folders.Current != null) list.Add(folders.Current);
                list.AddRange(folders.PinnedItems.Where(pf => pf != null));
                return (IEnumerable<FolderTreeItemViewModel>)list.Distinct(); 
            })
            .ToProperty(this, vm => vm.AllFoldersTracked, initialValue: Enumerable.Empty<FolderTreeItemViewModel>(), scheduler: RxApp.MainThreadScheduler);

        Pin = ReactiveCommand.CreateFromTask(async () =>
        {
            try
            {
                var folderToPinPath = await SelectFolder.Handle(Unit.Default);

                if (string.IsNullOrEmpty(folderToPinPath) || _pinnedFoldersSourceList.Items.Any(f => f.Path.PathEquals(folderToPinPath))) return;

                _pinnedFoldersSourceList.Add(
                    new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler) 
                    {
                        Path = folderToPinPath 
                    });
            }
            catch (UnhandledInteractionException<Unit, string?>) { /* User cancelled */ }
        });

        var canPinSelectedExecute = this.WhenAnyValue(x => x.Selected)
            .Select(s => s != null && !_pinnedFoldersBinding.Any(f => f.Path.PathEquals(s.Path)));

        PinSelected = ReactiveCommand.Create(() =>
        {
            if (Selected != null && !_pinnedFoldersSourceList.Items.Contains(Selected)) // Ensure not already pinned
            {
                 _pinnedFoldersSourceList.Add(Selected);
            }
        }, canPinSelectedExecute);

        var canUnpinSelectedExecute = this.WhenAnyValue(vm => vm.Selected)
            .Select(s => s != null && _pinnedFoldersBinding.Any(f => f.Path.PathEquals(s.Path)));

        UnpinSelected = ReactiveCommand.Create(() =>
        {
            if (Selected == null) return;
            var pinned = _pinnedFoldersSourceList.Items.FirstOrDefault(f => f.Path.PathEquals(Selected.Path));
            if (pinned != null) _pinnedFoldersSourceList.Remove(pinned);
        }, canUnpinSelectedExecute);

        var canMovePinnedFolderUp = this.WhenAnyValue(x => x.Selected)
            .Select(s => s != null && _pinnedFoldersSourceList.Items.Contains(s) && _pinnedFoldersSourceList.Items.IndexOf(s) > 0);

        MoveSelectedPinnedFolderUp = ReactiveCommand.Create(() =>
        {
            if (Selected == null) return;
            var pinnedIndex = _pinnedFoldersSourceList.Items.IndexOf(Selected);
            if (pinnedIndex > 0) _pinnedFoldersSourceList.Move(pinnedIndex, pinnedIndex - 1);
        }, canMovePinnedFolderUp);

        var canMovePinnedFolderDown = this.WhenAnyValue(x => x.Selected)
            .Select(s => s != null && _pinnedFoldersSourceList.Items.Contains(s) && _pinnedFoldersSourceList.Items.IndexOf(s) < _pinnedFoldersSourceList.Count - 1);

        MoveSelectedPinnedFolderDown = ReactiveCommand.Create(() =>
        {
            if (Selected == null) return;
            var pinnedIndex = _pinnedFoldersSourceList.Items.IndexOf(Selected);
            if (pinnedIndex < _pinnedFoldersSourceList.Count - 1) _pinnedFoldersSourceList.Move(pinnedIndex, pinnedIndex + 1);
        }, canMovePinnedFolderDown);

        CreateFolderUnderSelected = ReactiveCommand.CreateFromTask(async () =>
        {
            if (Selected == null) return; 
            var name = await PromptForName.Handle(Unit.Default);
            if (string.IsNullOrEmpty(name)) return;
            await Selected.CreateFolderCommand.Execute(name);
        }, this.WhenAnyValue(x => x.Selected).Select(s => s != null && !s.IsPlaceholder)); // Can't create under placeholder

        SelectNextFolder = ReactiveCommand.Create(() => 
        {
            if (AllFoldersTracked == null || !AllFoldersTracked.Any()) return;
            var currentList = AllFoldersTracked.ToList(); 
            var currentIndex = Selected != null ? currentList.IndexOf(Selected) : -1;
            Selected = currentList.ElementAtOrDefault((currentIndex + 1) % currentList.Count);
        });

        SelectPreviousFolder = ReactiveCommand.Create(() => 
        {
            if (AllFoldersTracked == null || !AllFoldersTracked.Any()) return;
            var currentList = AllFoldersTracked.ToList();
            var currentIndex = Selected != null ? currentList.IndexOf(Selected) : -1;
            Selected = currentList.ElementAtOrDefault((currentIndex - 1 + currentList.Count) % currentList.Count);
        });

        ExpandSelected = ReactiveCommand.Create(() => 
        {
            if (Selected != null) Selected.IsExpanded = true;
        }, this.WhenAnyValue(x => x.Selected).Select(s => s != null && !s.IsExpanded && !s.IsPlaceholder));

        CollapseSelectedOrGoToParent = ReactiveCommand.Create(() => 
        {
            if (Selected != null)
            {
                if (Selected.IsExpanded && Selected.Children.Any()) Selected.IsExpanded = false;
                else if (Selected.Parent != null) Selected = Selected.Parent;
            }
        }, this.WhenAnyValue(x => x.Selected).Select(s => s != null && ((s.IsExpanded && s.Children.Any()) || s.Parent != null)));

        SetSelectedFolderAsCurrentImplicitly = ReactiveCommand.Create(() => 
        {
            if (Selected != null && !Selected.IsPlaceholder) CurrentFolder = Selected;
        }, this.WhenAnyValue(x => x.Selected).Select(s => s != null && s != CurrentFolder && !s.IsPlaceholder));

        PinCurrentFolder = ReactiveCommand.Create(() => 
        {
            if (CurrentFolder != null && !_pinnedFoldersSourceList.Items.Contains(CurrentFolder))
            {
                _pinnedFoldersSourceList.Add(CurrentFolder);
            }
        }, this.WhenAnyValue(x => x.CurrentFolder).Select(cf => cf != null && !_pinnedFoldersSourceList.Items.Contains(cf)));

        FolderTreeItemViewModel? oldFolder = null; // Nullable

        this.WhenAnyValue(x => x.CurrentFolder)
            .Subscribe(f => // f can be null
            {
                if (oldFolder != null) oldFolder.IsCurrentFolder = false;
                if (f != null) f.IsCurrentFolder = true;
                oldFolder = f;
            });
        
        GoToParentFolderCommand = ReactiveCommand.Create(() => 
        {
            if (CurrentFolder?.Parent != null) CurrentFolder = CurrentFolder.Parent;
        }, this.WhenAnyValue(x => x.CurrentFolder).Select(cf => cf?.Parent != null));

        OpenSelectedFolderCommand = ReactiveCommand.Create(() => 
        {
            if (Selected != null && !Selected.IsPlaceholder) CurrentFolder = Selected;
        }, this.WhenAnyValue(x => x.Selected).Select(s => s != null && !s.IsPlaceholder));
    }

    public void AddPinnedFoldersFromPaths(IEnumerable<string> paths)
    {
        _pinnedFoldersSourceList.AddRange(paths
            .Where(p => !string.IsNullOrEmpty(p) && _fileSystem.DirectoryExists(p))
            .Select(p => new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler) { Path = p })
            .Where(f => f != null && !_pinnedFoldersSourceList.Items.Any(existing => existing.Path.PathEquals(f.Path))) // Ensure not already pinned
            .ToList()); // ToList to avoid issues with modifying collection while iterating (though AddRange might handle it)
    }

    ~FoldersViewModel()
    {
        _pinnedFoldersSourceList.Dispose();
    }
}