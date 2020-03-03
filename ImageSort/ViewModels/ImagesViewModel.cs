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

namespace ImageSort.ViewModels
{
    public class ImagesViewModel : ReactiveObject
    {
        private static readonly string[] supportedTypes = new[] { ".png", ".jpg", ".gif", ".bmp", ".tiff", ".tif", ".ico" };
        private FileSystemWatcher folderWatcher;

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

        public Interaction<Unit, string> PromptForNewFileName { get; }
            = new Interaction<Unit, string>();

        public ReactiveCommand<Unit, Unit> GoLeft { get; }
        public ReactiveCommand<Unit, Unit> GoRight { get; }
        public ReactiveCommand<Unit, Unit> RenameImage { get; }

        public ImagesViewModel(IFileSystem fileSystem = null, Func<FileSystemWatcher> folderWatcherFactory = null)
        {
            fileSystem ??= Locator.Current.GetService<IFileSystem>();
            folderWatcherFactory ??= () => Locator.Current.GetService<FileSystemWatcher>();

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

            images.Connect()
                .Subscribe(_ =>
                {
                    // necessary to notice the update
                    if (SelectedIndex == 0) SelectedIndex = -1;

                    if (SelectedIndex < 0) SelectedIndex = 0;
                });

            var canGoLeft = this.WhenAnyValue(x => x.SelectedIndex, x => x.Images.Count, (i, _) => i)
                .Select(i => 0 < i);

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

            var canRenameImage = this.WhenAnyValue(x => x.SelectedImage)
                .Select(p => !string.IsNullOrEmpty(p));

            RenameImage = ReactiveCommand.CreateFromTask(async () =>
            {
                var newFileName = await PromptForNewFileName.Handle(Unit.Default);

                if (newFileName != null)
                {
                    if (newFileName.Contains(@"\", StringComparison.OrdinalIgnoreCase) 
                        || newFileName.Contains("/", StringComparison.OrdinalIgnoreCase)
                        || newFileName.IndexOfAny(Path.GetInvalidPathChars()) >= 0) 
                        return;

                    var newPath = Path.Combine(CurrentFolder, newFileName + Path.GetExtension(SelectedImage));

                    var selectedImage = SelectedImage;

                    images.Replace(selectedImage, newPath);

                    fileSystem.Move(selectedImage, newPath);
                }
            }, canRenameImage);
        }


        public void RemoveImage(string image)
        {
            images.Remove(image);
        }

        public void InsertImage(string image)
        {
            images.Add(image);
        }

        private void OnImageCreated(object sender, FileSystemEventArgs e)
        {
            if (e.FullPath.EndsWithAny(StringComparison.OrdinalIgnoreCase, supportedTypes))
            {
                RxApp.MainThreadScheduler.Schedule(() => images.Add(e.FullPath));
            }
        }

        private void OnImageDeleted(object sender, FileSystemEventArgs e)
        {
            var item = images.Items.FirstOrDefault(i => i == e.FullPath);

            if (item != null)
            {
                RxApp.MainThreadScheduler.Schedule(() => images.Remove(item));
            }
        }

        private void OnImageRenamed(object sender, RenamedEventArgs e)
        {
            var item = images.Items.FirstOrDefault(i => i == e.OldFullPath);

            if (item != null)
            {
                RxApp.MainThreadScheduler.Schedule(() =>
                {
                    images.Remove(item);

                    images.Add(e.FullPath);
                });
            }
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
}