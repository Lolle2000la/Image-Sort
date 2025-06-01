using DynamicData;
using DynamicData.Binding;
using ImageSort.Actions;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using ImageSort.Localization;
using ReactiveUI;
using Splat;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using ImageSort.ViewModels.Metadata; // Added using
using ImageSort.Actions; // Required for DeleteAction

namespace ImageSort.ViewModels;

public class ImagesViewModel : ReactiveObject
{
    private static readonly string[] supportedTypes = new[] { ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".tif", ".ico", ".webp" };
    private FileSystemWatcher folderWatcher;
    private readonly IFileSystem fileSystem; // Store IFileSystem
    private readonly IRecycleBin recycleBin; // Store IRecycleBin

    private string _currentPath;

    public string CurrentFolder
    {
        get => _currentPath;
        set => this.RaiseAndSetIfChanged(ref _currentPath, value);
    }

    private readonly SourceList<string> images;

    private readonly ReadOnlyObservableCollection<string> _images;
    public ReadOnlyObservableCollection<string> Images => _images;

    private int _selectedIndex;

    public int SelectedIndex
    {
        get => _selectedIndex;
        set => this.RaiseAndSetIfChanged(ref _selectedIndex, value);
    }

    private readonly ObservableAsPropertyHelper<string> _selectedImage;
    public string SelectedImage => _selectedImage.Value;

    private string _searchTerm;

    public string SearchTerm
    {
        get => _searchTerm;
        set => this.RaiseAndSetIfChanged(ref _searchTerm, value);
    }

    private bool _isMetadataVisible = true; // Default to true or as per design
    public bool IsMetadataVisible
    {
        get => _isMetadataVisible;
        set => this.RaiseAndSetIfChanged(ref _isMetadataVisible, value);
    }

    public Interaction<Unit, string> PromptForNewFileName { get; }
        = new Interaction<Unit, string>();

    public Interaction<string, Unit> NotifyUserOfError { get; }
        = new Interaction<string, Unit>();

    public ReactiveCommand<Unit, Unit> GoLeft { get; }
    public ReactiveCommand<Unit, Unit> GoRight { get; }
    public ReactiveCommand<Unit, IReversibleAction> RenameImage { get; }

    // Add new commands based on AppAction
    public ReactiveCommand<Unit, Unit> SelectNextImage { get; }
    public ReactiveCommand<Unit, Unit> SelectPreviousImage { get; }
    public ReactiveCommand<Unit, IReversibleAction> DeleteImageCommand { get; } // To avoid conflict if a DeleteImage method exists

    public MetadataViewModel Metadata { get; } // Added Metadata property

    public ImagesViewModel(IFileSystem fileSystem = null, Func<FileSystemWatcher> folderWatcherFactory = null, 
                         IMetadataExtractor metadataExtractor = null, 
                         MetadataSectionViewModelFactory metadataSectionFactory = null,
                         IRecycleBin recycleBin = null) // Added IRecycleBin
    {
        this.fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();
        folderWatcherFactory ??= () => Locator.Current.GetService<FileSystemWatcher>();
        metadataExtractor ??= Locator.Current.GetService<IMetadataExtractor>(); // Resolve IMetadataExtractor
        metadataSectionFactory ??= Locator.Current.GetService<MetadataSectionViewModelFactory>(); // Resolve MetadataSectionViewModelFactory
        this.recycleBin = recycleBin ?? Locator.Current.GetService<IRecycleBin>(); // Resolve and store IRecycleBin

        Metadata = new MetadataViewModel(metadataExtractor, this.fileSystem, metadataSectionFactory); // Initialize MetadataViewModel

        images = new SourceList<string>();

        images.Connect()
            .Filter(this.WhenAnyValue(x => x.SearchTerm)
                .Select<string, Func<string, bool>>(t => p => t == null || p.Contains(t, StringComparison.OrdinalIgnoreCase)))
            .Sort(SortExpressionComparer<string>.Ascending(p => p))
            .Bind(out _images)
            .Subscribe();

        this.WhenAnyValue(x => x.CurrentFolder)
            .Where(f => f != null)
            .Select(f => fileSystem.GetFiles(f)
                                  .Where(s => s.EndsWithAny(
                                      StringComparison.OrdinalIgnoreCase,
                                      supportedTypes)))
            .Subscribe(i =>
            {
                images.Clear();

                images.AddRange(i);
            });

        _selectedImage = this.WhenAnyValue(x => x.SelectedIndex)
            .Select(i => Images.ElementAtOrDefault(i))
            .ToProperty(this, x => x.SelectedImage);

        // Update Metadata.ImagePath when SelectedImage changes
        this.WhenAnyValue(x => x.SelectedImage)
            .Subscribe(path => 
            {
                if (Metadata != null)
                {
                    Metadata.ImagePath = path;
                }
            });

        images.Connect()
            .Subscribe(_ =>
            {
                // necessary to notice the update
                if (SelectedIndex == 0) SelectedIndex = -1;

                if (SelectedIndex < 0) SelectedIndex = 0;
            });

        var canGoLeft = this.WhenAnyValue(x => x.SelectedIndex, x => x.Images.Count, (i, _) => i)
            .Select(i => i > 0);

        GoLeft = ReactiveCommand.Create(() =>
        {
            SelectedIndex--;
        }, canGoLeft);

        var canGoRight = this.WhenAnyValue(x => x.SelectedIndex, x => x.Images.Count, (i, _) => i)
            .Select(i => i < Images.Count - 1);

        GoRight = ReactiveCommand.Create(() =>
        {
            SelectedIndex++;
        }, canGoRight);

        SelectNextImage = GoRight; // Alias GoRight
        SelectPreviousImage = GoLeft; // Alias GoLeft

        var canRenameImage = this.WhenAnyValue(x => x.SelectedImage)
            .Select(p => !string.IsNullOrEmpty(p));

        RenameImage = ReactiveCommand.CreateFromTask<IReversibleAction>(async _ =>
        {
            var newFileName = await PromptForNewFileName.Handle(Unit.Default);

            if (!string.IsNullOrEmpty(newFileName))
            {
                if (newFileName.Contains(@"\", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("/", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("*", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("?", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains(":", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("<", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains(">", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("|", StringComparison.OrdinalIgnoreCase)
                    || newFileName.Contains("\"", StringComparison.OrdinalIgnoreCase)
                    || newFileName.IndexOfAny(Path.GetInvalidPathChars()) >= 0)
                {
                    await NotifyUserOfError.Handle(Text.RenameNewNameContainsIllegalCharacters
                        .Replace("{FileName}", newFileName, StringComparison.OrdinalIgnoreCase));

                    return null;
                }

                try
                {
                    return new RenameAction(SelectedImage, newFileName, fileSystem,
                        (o, n) => images.Replace(o, n), (n, o) => images.Replace(n, o));
                }
                catch (IOException ex)
                {
                    await NotifyUserOfError.Handle(ex.Message);
                }
            }

            return null;
        }, canRenameImage);

        var canDeleteImage = this.WhenAnyValue(x => x.SelectedImage)
            .Select(p => !string.IsNullOrEmpty(p));

        DeleteImageCommand = ReactiveCommand.Create<IReversibleAction>(() =>
        {
            if (string.IsNullOrEmpty(SelectedImage)) return null;

            try
            {
                return new DeleteAction(SelectedImage, this.fileSystem, this.recycleBin,
                    path => images.Remove(path), // onAct: remove from UI
                    path => images.Add(path)     // onRevert: add back to UI, list will re-sort
                );
            }
            catch (Exception ex)
            {
                NotifyUserOfError.Handle(ex.Message).Subscribe();
                return null;
            }
        }, canDeleteImage);


        this.WhenAnyValue(x => x.CurrentFolder)
            .Where(f => !string.IsNullOrEmpty(f))
            .Subscribe(f =>
            {
                folderWatcher?.Dispose();
                folderWatcher = folderWatcherFactory();

                if (folderWatcher == null) return;

                folderWatcher.Path = f;
                folderWatcher.IncludeSubdirectories = false;
                folderWatcher.NotifyFilter = NotifyFilters.FileName;
                folderWatcher.InternalBufferSize = 64000;
                folderWatcher.EnableRaisingEvents = true;

                folderWatcher.Created += OnImageCreated;
                folderWatcher.Deleted += OnImageDeleted;
                folderWatcher.Renamed += OnImageRenamed;
            });
    }

    public void RemoveImage(string image)
    {
        images.Remove(image);
    }

    public void InsertImage(string image)
    {
        images.Add(image);
    }

    public void OnImageMoved(string oldPath, string newPath)
    {
        if (images.Items.Contains(oldPath))
        {
            images.Replace(oldPath, newPath);
        }
        else
        {
            // If the old path wasn't tracked (e.g., image moved from outside current folder into a subfolder)
            // and the new path is in the current folder, add it.
            // This logic might need refinement based on how external moves are handled.
            // For now, primarily for moves initiated by the app.
        }
    }

    private void OnImageCreated(object sender, FileSystemEventArgs e)
    {
        RxApp.MainThreadScheduler.Schedule(() =>
        {
            if (e.FullPath.EndsWithAny(StringComparison.OrdinalIgnoreCase, supportedTypes)
                && !images.Items.Contains(e.FullPath))
            {
                images.Add(e.FullPath);
            }
        });
    }

    private void OnImageDeleted(object sender, FileSystemEventArgs e)
    {
        var item = images.Items.FirstOrDefault(i => i.PathEquals(e.FullPath));

        RxApp.MainThreadScheduler.Schedule(() =>
        {
            if (item != null && images.Items.Contains(item))
            {
                images.Remove(item);
            }
        });
    }

    private void OnImageRenamed(object sender, RenamedEventArgs e)
    {
        var item = images.Items.FirstOrDefault(i => i.PathEquals(e.OldFullPath));

        RxApp.MainThreadScheduler.Schedule(() =>
        {
            if (item != null && images.Items.Contains(item))
            {
                images.Replace(item, e.FullPath);
            }
        });
    }

    ~ImagesViewModel()
    {
        if (folderWatcher != null)
        {
            folderWatcher.Created -= OnImageCreated;
            folderWatcher.Deleted -= OnImageDeleted;
            folderWatcher.Renamed -= OnImageRenamed;
            folderWatcher.Dispose();
        }

        images.Dispose();
    }
}