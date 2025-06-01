#nullable enable

using DynamicData;
using ImageSort.FileSystem;
using ImageSort.Helpers; // Added for PathEquals
using ReactiveUI;
using Splat;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Threading.Tasks;
using System.Collections.Generic; 

namespace ImageSort.ViewModels;

public class FolderTreeItemViewModel : ReactiveObject, IDisposable
{
    private readonly CompositeDisposable disposableRegistration = new CompositeDisposable();
    private readonly IFileSystem _fileSystem; 
    private readonly IScheduler _backgroundScheduler; 
    private readonly Func<FileSystemWatcher> _folderWatcherFactory; 
    private readonly FileSystemWatcher? _folderWatcher; 

    private bool _isExpanded;
    public bool IsExpanded
    {
        get => _isExpanded;
        set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
    }

    private bool _childrenLoaded = false;

    public bool IsPlaceholder { get; init; } = false;

    private bool _isSelected;
    public bool IsSelected
    {
        get => _isSelected;
        set => this.RaiseAndSetIfChanged(ref _isSelected, value);
    }

    private bool _isCurrentFolder;
    public bool IsCurrentFolder
    {
        get => _isCurrentFolder;
        set => this.RaiseAndSetIfChanged(ref _isCurrentFolder, value);
    }

    private bool _isVisible = true; 
    public bool IsVisible
    {
        get => _isVisible;
        set => this.RaiseAndSetIfChanged(ref _isVisible, value);
    }

    private string _path = string.Empty;
    public string Path
    {
        get => _path;
        set => this.RaiseAndSetIfChanged(ref _path, value); 
    }

    private readonly ObservableAsPropertyHelper<string> _folderNameOaph;
    public string FolderName => _folderNameOaph.Value;

    private readonly SourceList<FolderTreeItemViewModel> _subFoldersSourceList; 

    private readonly ReadOnlyObservableCollection<FolderTreeItemViewModel> _childrenBinding;
    public ReadOnlyObservableCollection<FolderTreeItemViewModel> Children => _childrenBinding;

    public FolderTreeItemViewModel? Parent { get; } 

    public ReactiveCommand<string, FolderTreeItemViewModel?> CreateFolderCommand { get; }

    // Primary constructor
    public FolderTreeItemViewModel(
        IFileSystem fileSystem, // Made non-nullable for the main constructor path
        Func<FileSystemWatcher> folderWatcherFactory, // Made non-nullable
        IScheduler backgroundScheduler, // Made non-nullable
        FolderTreeItemViewModel? parent = null)
    {
        Parent = parent;
        _fileSystem = fileSystem; // Direct assignment
        _backgroundScheduler = backgroundScheduler;
        _folderWatcherFactory = folderWatcherFactory;
        
        _folderWatcher = _folderWatcherFactory.Invoke();
        _folderWatcher?.DisposeWith(disposableRegistration);

        _subFoldersSourceList = new SourceList<FolderTreeItemViewModel>().DisposeWith(disposableRegistration);
        _subFoldersSourceList.Connect()
            .Sort(Comparer<FolderTreeItemViewModel>.Create((a, b) =>
            {
                if (a.IsPlaceholder && !b.IsPlaceholder) return -1; 
                if (!a.IsPlaceholder && b.IsPlaceholder) return 1;
                return string.Compare(a.Path, b.Path, StringComparison.OrdinalIgnoreCase);
            }))
            .ObserveOn(RxApp.MainThreadScheduler)
            .Bind(out _childrenBinding)
            .Subscribe()
            .DisposeWith(disposableRegistration);

        this.WhenAnyValue(x => x.Path)
            .Where(p => !string.IsNullOrEmpty(p) && !_childrenLoaded && !IsPlaceholder)
            .ObserveOn(_backgroundScheduler)
            .Subscribe(async p =>
            {
                bool hasAnySubfolders = false;
                try
                {
                    hasAnySubfolders = await Task.Run(() => _fileSystem.GetSubFolders(p!).Any());
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Error checking for subfolders in {p} for placeholder: {ex.Message}");
                }
                RxApp.MainThreadScheduler.Schedule(() =>
                {
                    if (hasAnySubfolders && !_childrenLoaded && _subFoldersSourceList.Count == 0)
                    {
                        _subFoldersSourceList.Add(new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler, this) { IsPlaceholder = true, Path = System.IO.Path.Combine(p!, "placeholder") });
                    }
                });
            })
            .DisposeWith(disposableRegistration);

        _folderNameOaph = this.WhenAnyValue(x => x.Path)
            .Select(p =>
            {
                if (string.IsNullOrEmpty(p)) return string.Empty;
                if (IsPlaceholder) return "(loading...)"; 

                string name = System.IO.Path.GetFileName(p);
                if (string.IsNullOrEmpty(name))
                {
                    string tempPath = p.TrimEnd(System.IO.Path.DirectorySeparatorChar, System.IO.Path.AltDirectorySeparatorChar);
                    if (string.IsNullOrEmpty(tempPath)) return p;
                    name = System.IO.Path.GetFileName(tempPath);
                    if (string.IsNullOrEmpty(name)) return p;
                }
                return name;
            })
            .ToProperty(this, x => x.FolderName, initialValue: string.Empty, scheduler: RxApp.MainThreadScheduler)
            .DisposeWith(disposableRegistration);

        this.WhenAnyValue(x => x.IsExpanded, x => x.Path)
            .Where(x => x.Item1 && !_childrenLoaded && !string.IsNullOrEmpty(x.Item2) && !IsPlaceholder)
            .ObserveOn(_backgroundScheduler)
            .Select(x => x.Item2)
            .Subscribe(async path => await LoadChildrenAsync(path!))
            .DisposeWith(disposableRegistration);

        if (_folderWatcher != null)
        {
            this.WhenAnyValue(x => x.Path)
                .Where(p => !string.IsNullOrEmpty(p) && !IsPlaceholder) 
                .ObserveOn(RxApp.MainThreadScheduler) 
                .Subscribe(p =>
                {
                    try
                    {
                        _folderWatcher.Path = p!;
                        _folderWatcher.IncludeSubdirectories = false;
                        _folderWatcher.EnableRaisingEvents = true;
                    }
                    catch (Exception ex)
                    {
                        System.Diagnostics.Debug.WriteLine($"Error setting up watcher for {p}: {ex.Message}");
                    }
                })
                .DisposeWith(disposableRegistration);

            Observable.FromEventPattern<FileSystemEventHandler, FileSystemEventArgs>(
                h => _folderWatcher.Created += h, h => _folderWatcher.Created -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => AddSubFolder(e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);

            Observable.FromEventPattern<FileSystemEventHandler, FileSystemEventArgs>(
                h => _folderWatcher.Deleted += h, h => _folderWatcher.Deleted -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => RemoveSubFolder(e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);

            Observable.FromEventPattern<RenamedEventHandler, RenamedEventArgs>(
                h => _folderWatcher.Renamed += h, h => _folderWatcher.Renamed -= h)
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(e => RenameSubFolder(e.EventArgs.OldFullPath, e.EventArgs.FullPath))
                .DisposeWith(disposableRegistration);
        }

        CreateFolderCommand = ReactiveCommand.Create<string, FolderTreeItemViewModel?>(
            (name) => CreateFolderInternal(name),
            this.WhenAnyValue(x => x.Path).Select(p => !string.IsNullOrEmpty(p) && !IsPlaceholder) 
        ).DisposeWith(disposableRegistration);
    }

    // Constructor for XAML Designer / Service Locator fallback
    public FolderTreeItemViewModel() : this(
        Locator.Current.GetService<IFileSystem>() ?? new DesignTimeFileSystem(), // Use a specific design-time FS
        () => Locator.Current.GetService<FileSystemWatcher>() ?? new FileSystemWatcher(), // Provide a factory that can return a default
        RxApp.MainThreadScheduler, 
        null) 
    {
        IsVisible = true;
        Path = @"C:\Design"; 
        IsExpanded = true;

        var child1 = new FolderTreeItemViewModel(this._fileSystem, this._folderWatcherFactory, this._backgroundScheduler, this) { Path = @"C:\Design\Child1", IsVisible = true };
        var child2 = new FolderTreeItemViewModel(this._fileSystem, this._folderWatcherFactory, this._backgroundScheduler, this) { Path = @"C:\Design\Child2", IsVisible = true };
        _subFoldersSourceList.AddRange(new[] { child1, child2 });
    }

    private async Task LoadChildrenAsync(string path)
    {
        if (IsPlaceholder || _childrenLoaded) return;

        _subFoldersSourceList.Edit(update => update.Clear()); 

        if (string.IsNullOrEmpty(path))
        {
            _childrenLoaded = true;
            return;
        }

        try
        {
            var newSubFolders = (await Task.Run(() => _fileSystem.GetSubFolders(path).ToList())) 
                .Select(p => new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler, this) { Path = p, IsVisible = true });
            
            _subFoldersSourceList.AddRange(newSubFolders);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Error loading children for {path}: {ex.Message}");
        }
        finally
        {
            _childrenLoaded = true;
        }
    }

    private void AddSubFolder(string path)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            if (_fileSystem.DirectoryExists(path) && !_subFoldersSourceList.Items.Any(f => !f.IsPlaceholder && f.Path.PathEquals(path)))
            {
                if (_childrenLoaded) 
                {
                    var placeholder = _subFoldersSourceList.Items.FirstOrDefault(i => i.IsPlaceholder);
                    if (placeholder != null) _subFoldersSourceList.Remove(placeholder);

                    _subFoldersSourceList.Add(new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler, this) { Path = path, IsVisible = true });
                }
            }
        });
    }

    private void RemoveSubFolder(string path)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            var existing = _subFoldersSourceList.Items.FirstOrDefault(f => !f.IsPlaceholder && f.Path.PathEquals(path));
            if (existing != null) 
            {
                _subFoldersSourceList.Remove(existing);
                existing.Dispose();
            }
            if (!_subFoldersSourceList.Items.Any(i => !i.IsPlaceholder) && _childrenLoaded && !IsPlaceholder)
            {
                _backgroundScheduler.Schedule(async () => {
                    bool hasAnySubfolders = false;
                    try { hasAnySubfolders = await Task.Run(() => _fileSystem.GetSubFolders(this.Path).Any()); } catch { /* ignore */ }
                    if (hasAnySubfolders) {
                        RxApp.MainThreadScheduler.Schedule(() => {
                             if (!_subFoldersSourceList.Items.Any())
                             {
                                _subFoldersSourceList.Add(new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler, this) { IsPlaceholder = true, Path = System.IO.Path.Combine(this.Path, "placeholder") });
                             }
                        });
                    }
                });
            }
        });
    }

    private void RenameSubFolder(string oldPath, string newPath)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            var existing = _subFoldersSourceList.Items.FirstOrDefault(f => !f.IsPlaceholder && f.Path.PathEquals(oldPath));
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

    private FolderTreeItemViewModel? CreateFolderInternal(string name)
    {
        if (string.IsNullOrWhiteSpace(name) || IsPlaceholder)
        {
            System.Diagnostics.Debug.WriteLine("Folder name cannot be empty or called on a placeholder.");
            return null;
        }

        var newPath = System.IO.Path.Combine(Path, name);

        if (_fileSystem.DirectoryExists(newPath)) 
        {
            return _subFoldersSourceList.Items.FirstOrDefault(c => !c.IsPlaceholder && c.Path.PathEquals(newPath));
        }

        try
        {
            System.IO.Directory.CreateDirectory(newPath); 
            
            var newFolderVM = new FolderTreeItemViewModel(_fileSystem, _folderWatcherFactory, _backgroundScheduler, this) { Path = newPath, IsVisible = true };
            if (!_subFoldersSourceList.Items.Any(f => f.Path.PathEquals(newPath))) 
            {
                 _subFoldersSourceList.Add(newFolderVM);
            }
            return newFolderVM;
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to create directory '{newPath}': {ex.Message}");
            return null;
        }
    }
    
    public void AddChild(FolderTreeItemViewModel child) 
    {
        if (!_subFoldersSourceList.Items.Contains(child))
        {
            _subFoldersSourceList.Add(child);
        }
    }
    
    public void ClearChildren()
    {
        _subFoldersSourceList.Clear(); 
    }

    public void Dispose()
    {
        disposableRegistration.Dispose();
        foreach (var child in _subFoldersSourceList.Items.ToList()) 
        {
            child.Dispose();
        }
        _subFoldersSourceList.Clear(); 
    }
}

// Simple DesignTimeFileSystem to satisfy the constructor for the designer
internal class DesignTimeFileSystem : IFileSystem
{
    public IEnumerable<string> GetSubFolders(string path) { yield break; } // No subfolders in design time
    public IEnumerable<string> GetFiles(string folder) { yield break; } // No files
    public bool IsFolderEmpty(string path) => true;
    // FileExists, DirectoryExists, Move, CreateFolder use default interface implementations or are not critical for design view
}

#nullable disable