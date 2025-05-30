using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using ReactiveUI;
using Splat;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Threading.Tasks;
using System.Collections.Generic;

namespace ImageSort.ViewModels;

public class FolderTreeItemViewModel : ReactiveObject, IDisposable
{
    private readonly CompositeDisposable disposableRegistration = new CompositeDisposable();
    private readonly IFileSystem fileSystem;
    private readonly IScheduler backgroundScheduler;
    private readonly Func<FileSystemWatcher> folderWatcherFactory;
    private readonly FileSystemWatcher folderWatcher;

    private bool _isExpanded;
    public bool IsExpanded
    {
        get => _isExpanded;
        set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
    }

    private bool _childrenLoaded = false;

    public bool IsPlaceholder { get; init; } = false;

    private bool _isCurrentFolder;

    public bool IsCurrentFolder
    {
        get => _isCurrentFolder;
        set => this.RaiseAndSetIfChanged(ref _isCurrentFolder, value);
    }

    private bool _isVisible;

    public bool IsVisible
    {
        get => _isVisible;
        set => this.RaiseAndSetIfChanged(ref _isVisible, value);
    }

    private string _path;

    public string Path
    {
        get => _path;
        set => this.RaiseAndSetIfChanged(ref _path, value);
    }

    private readonly ObservableAsPropertyHelper<string> _folderName;
    public string FolderName => _folderName.Value;

    private readonly SourceList<FolderTreeItemViewModel> subFolders;

    private readonly ReadOnlyObservableCollection<FolderTreeItemViewModel> _children;
    public ReadOnlyObservableCollection<FolderTreeItemViewModel> Children => _children;

    public ReactiveCommand<string, Unit> CreateFolder { get; }

    public FolderTreeItemViewModel(IFileSystem fileSystem = null, Func<FileSystemWatcher> folderWatcherFactory = null, IScheduler backgroundScheduler = null)
    {
        this.fileSystem = fileSystem ??= Locator.Current.GetService<IFileSystem>();
        this.backgroundScheduler = backgroundScheduler ??= RxApp.TaskpoolScheduler;
        this.folderWatcherFactory = folderWatcherFactory ??= () => Locator.Current.GetService<FileSystemWatcher>();
        folderWatcher = this.folderWatcherFactory(); 
        folderWatcher?.DisposeWith(disposableRegistration);

        subFolders = new SourceList<FolderTreeItemViewModel>();
        subFolders.Connect()
            .Sort(Comparer<FolderTreeItemViewModel>.Create((a, b) =>
            {
                if (a == null && b == null) return 0;
                if (a == null) return -1; // Nulls first
                if (b == null) return 1;  // Real items after nulls
                // For actual ViewModels, sort by Path
                if (a.Path == null && b.Path == null) return 0;
                if (a.Path == null) return -1;
                if (b.Path == null) return 1;
                return string.Compare(a.Path, b.Path, StringComparison.OrdinalIgnoreCase);
            }))
            .ObserveOn(RxApp.MainThreadScheduler)
            .Bind(out _children)
            .Subscribe()
            .DisposeWith(disposableRegistration);

        subFolders.DisposeWith(disposableRegistration);

        // Conditionally add a placeholder item if subfolders exist, to enable the expander.
        // This will be cleared when children are actually loaded.
        this.WhenAnyValue(x => x.Path)
            .Where(p => !string.IsNullOrEmpty(p) && !_childrenLoaded)
            .ObserveOn(backgroundScheduler) // Perform filesystem check on background thread
            .Subscribe(async p =>
            {
                bool hasAnySubfolders = false;
                try
                {
                    hasAnySubfolders = await Task.Run(() => this.fileSystem.GetSubFolders(p).Any());
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Error checking for subfolders in {p} for placeholder: {ex.Message}");
                }

                // Switch back to main thread to modify subFolders collection if needed
                RxApp.MainThreadScheduler.Schedule(() =>
                {
                    if (hasAnySubfolders && !_childrenLoaded && subFolders.Count == 0 && !IsPlaceholder) // Don't add placeholder to a placeholder
                    {
                        subFolders.Add(new FolderTreeItemViewModel(this.fileSystem, this.folderWatcherFactory, this.backgroundScheduler) { IsPlaceholder = true }); // Add placeholder
                    }
                });
            })
            .DisposeWith(disposableRegistration);

        _folderName = this.WhenAnyValue(x => x.Path)
            .Select(p =>
            {
                var path = System.IO.Path.GetFileName(p);

                return string.IsNullOrEmpty(path) ? p : path; // on a disk path (e.g. C:\, Path.GetFileName() returns an empty string
            })
            .ToProperty(this, x => x.FolderName);

        // Load children when expanded for the first time and path is valid
        this.WhenAnyValue(x => x.IsExpanded, x => x.Path)
            .Where(x => x.Item1 && !_childrenLoaded && !string.IsNullOrEmpty(x.Item2))
            .ObserveOn(backgroundScheduler)
            .Select(x => x.Item2) // Select the path
            .Subscribe(async path =>
            {
                await LoadChildrenAsync(path);
                // _childrenLoaded is set within LoadChildrenAsync
            })
            .DisposeWith(disposableRegistration);

        // Setup FileSystemWatcher when Path changes
        this.WhenAnyValue(x => x.Path)
            .Where(p => !string.IsNullOrEmpty(p) && folderWatcher != null)
            .ObserveOn(RxApp.MainThreadScheduler) // Watcher setup might need main thread if it interacts with UI state directly
            .Subscribe(p =>
            {
                try
                {
                    folderWatcher.Path = p;
                    folderWatcher.IncludeSubdirectories = false; // Only watch direct children
                    folderWatcher.EnableRaisingEvents = true;
                }
                catch (Exception ex) // Catch potential errors during watcher setup (e.g. path not found)
                {
                    System.Diagnostics.Debug.WriteLine($"Error setting up watcher for {p}: {ex.Message}");
                }
            })
            .DisposeWith(disposableRegistration);

        if (folderWatcher != null)
        {
            Observable.FromEventPattern<FileSystemEventHandler, FileSystemEventArgs>(
                h => folderWatcher.Created += h, h => folderWatcher.Created -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => AddSubFolder(e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);

            Observable.FromEventPattern<FileSystemEventHandler, FileSystemEventArgs>(
                h => folderWatcher.Deleted += h, h => folderWatcher.Deleted -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => RemoveSubFolder(e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);

            Observable.FromEventPattern<RenamedEventHandler, RenamedEventArgs>(
                h => folderWatcher.Renamed += h, h => folderWatcher.Renamed -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => RenameSubFolder(e.EventArgs.OldFullPath, e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);
        }

        CreateFolder = ReactiveCommand.CreateFromTask<string, Unit>(async (name) =>
        {
            var newPath = System.IO.Path.Combine(Path, name);
            await Task.Run(() => fileSystem.CreateFolder(newPath));
            return Unit.Default;
        });

        CreateFolder.ThrownExceptions.Subscribe(ex => {
            // Log or handle folder creation errors
            System.Diagnostics.Debug.WriteLine($"Error creating folder: {ex.Message}");
        }).DisposeWith(disposableRegistration);
    }

    private async Task LoadChildrenAsync(string path)
    {
        if (IsPlaceholder) // Do not load children for a placeholder itself
        {
            _childrenLoaded = true; // Mark as loaded to prevent further attempts on this placeholder
            return;
        }

        if (_childrenLoaded && !string.IsNullOrEmpty(path)) // If already loaded for this path, don't reload unless forced
        {
            // Potentially add logic here if re-loading is ever needed.
            // For now, if _childrenLoaded is true, we assume content is up-to-date or handled by FileSystemWatcher.
            return;
        }

        // Clear existing items (including the placeholder) before loading new ones
        // This ensures that if children are removed, the UI updates correctly.
        subFolders.Edit(update => 
        {
            update.Clear();
        });

        if (string.IsNullOrEmpty(path))
        {
            _childrenLoaded = true; // Mark as loaded even if path is empty/invalid
            return;
        }

        try
        {
            var newSubFolders = (await Task.Run(() => fileSystem.GetSubFolders(path)))
                .Select(p => new FolderTreeItemViewModel(fileSystem, folderWatcherFactory, backgroundScheduler) { Path = p, IsVisible = true });
            
            subFolders.AddRange(newSubFolders); // Add the actual children
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Error loading children for {path}: {ex.Message}");
        }
        finally
        {
            _childrenLoaded = true; // Mark as loaded regardless of success or failure to prevent re-loading
        }
    }

    private void AddSubFolder(string path)
    {
        // Ensure this runs on the main thread if subFolders modification needs it
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            if (fileSystem.DirectoryExists(path) && !subFolders.Items.Any(f => f != null && !f.IsPlaceholder && f.Path.PathEquals(path)))
            {
                if (_childrenLoaded) 
                {
                    // Remove placeholder if it exists and we are adding a real item
                    var placeholder = subFolders.Items.FirstOrDefault(i => i != null && i.IsPlaceholder);
                    if (placeholder != null) subFolders.Remove(placeholder);

                    subFolders.Add(new FolderTreeItemViewModel(fileSystem, folderWatcherFactory, backgroundScheduler) { Path = path, IsVisible = true });
                }
                // If not _childrenLoaded, expansion will handle it.
                // Or, if a placeholder was needed, the creation logic for the parent should handle it.
            }
        });
    }

    private void RemoveSubFolder(string path)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            var existing = subFolders.Items.FirstOrDefault(f => f != null && !f.IsPlaceholder && f.Path.PathEquals(path));
            if (existing != null) 
            {
                subFolders.Remove(existing);
                existing.Dispose(); // Clean up the child ViewModel
            }

            // If all real children are removed and a placeholder was appropriate, it should be re-added.
            // This might require re-evaluating the placeholder condition for the parent.
            // For now, focus on removal. Adding placeholder on empty is handled by initial load logic.
        });
    }

    private void RenameSubFolder(string oldPath, string newPath)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            var existing = subFolders.Items.FirstOrDefault(f => f != null && !f.IsPlaceholder && f.Path.PathEquals(oldPath));
            if (existing != null)
            {
                existing.Path = newPath; 
            }
            else 
            {
                if (_childrenLoaded) AddSubFolder(newPath);
            }
        });
    }

    public void Dispose()
    {
        disposableRegistration.Dispose();
        folderWatcher?.Dispose(); // Ensure watcher is disposed
        // Dispose children if any to prevent resource leaks from their watchers
        foreach (var child in Children.ToList()) // ToList to avoid modification issues if child.Dispose removes itself
        {
            child?.Dispose();
        }
        subFolders.Clear(); // Clear the source list
        subFolders.Dispose(); // Dispose the source list itself
    }
}