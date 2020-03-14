using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Threading.Tasks;

namespace ImageSort.ViewModels
{
    public class FolderTreeItemViewModel : ReactiveObject
    {
        private readonly CompositeDisposable disposableRegistration = new CompositeDisposable();
        private readonly IFileSystem fileSystem;
        private readonly IScheduler backgroundScheduler;
        private readonly Func<FileSystemWatcher> folderWatcherFactory;
        private readonly FileSystemWatcher folderWatcher;
        private readonly bool noParallel;

        private bool _isCurrentFolder = false;

        public bool IsCurrentFolder
        {
            get => _isCurrentFolder;
            set => this.RaiseAndSetIfChanged(ref _isCurrentFolder, value);
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

        public FolderTreeItemViewModel(IFileSystem fileSystem = null, Func<FileSystemWatcher> folderWatcherFactory = null, bool noParallel = false)
        {
            this.fileSystem = fileSystem ??= Locator.Current.GetService<IFileSystem>();
            this.backgroundScheduler = backgroundScheduler ??= RxApp.TaskpoolScheduler;
            this.noParallel = noParallel;
            this.folderWatcherFactory = folderWatcherFactory ??= () => Locator.Current.GetService<FileSystemWatcher>();
            folderWatcher = folderWatcherFactory();
            folderWatcher?.DisposeWith(disposableRegistration);

            subFolders = new SourceList<FolderTreeItemViewModel>();
            subFolders.Connect()
                .Sort(SortExpressionComparer<FolderTreeItemViewModel>.Ascending(f => f.Path))
                .Bind(out _children)
                .Subscribe()
                .DisposeWith(disposableRegistration);

            subFolders.DisposeWith(disposableRegistration);

            _folderName = this.WhenAnyValue(x => x.Path)
                .Select(p =>
                {
                    var path = System.IO.Path.GetFileName(p);

                    return string.IsNullOrEmpty(path) ? p : path; // on a disk path (e.g. C:\, Path.GetFileName() returns an empty string
                })
                .ToProperty(this, x => x.FolderName);

            this.WhenAnyValue(x => x.Path)
                .Where(p => p != null)
                .SelectMany(async p =>
                {
                    try
                    {
                        if (noParallel) return fileSystem.GetSubFolders(p);

                        return await Task.Run(() => fileSystem.GetSubFolders(p)).ConfigureAwait(false);
                    }
                    catch
                    {
                        return null;
                    }
                })
                .Where(p => p != null)
                .Select(paths =>
                {
                    return paths.Where(p => p != null)
                        .Select(p =>
                        {
                            try
                            {
                                return new FolderTreeItemViewModel(fileSystem, folderWatcherFactory, noParallel) { Path = p };
                            }
                            catch (UnauthorizedAccessException) { return null; }
                        })
                        .Where(f => f != null)
                        .ToList();
                })
                .ObserveOn(RxApp.MainThreadScheduler)
                .Subscribe(folders => subFolders.AddRange(folders))
                .DisposeWith(disposableRegistration);

            CreateFolder = ReactiveCommand.Create<string, Unit>(name =>
                {
                    var newFolderPath = System.IO.Path.Combine(Path, name);

                    if (Children.Select(f => f.Path).Any(s => s == newFolderPath)) return Unit.Default;

                    fileSystem.CreateFolder(newFolderPath);

                    subFolders.Add(new FolderTreeItemViewModel(fileSystem, noParallel: noParallel) { Path = newFolderPath });

                    return Unit.Default;
                });

            this.WhenAnyValue(x => x.Path)
                .Where(p => !string.IsNullOrEmpty(p))
                .Where(_ => folderWatcher != null)
                .Subscribe(p =>
                {
                    folderWatcher.Path = p;
                    folderWatcher.IncludeSubdirectories = false;
                    folderWatcher.NotifyFilter = NotifyFilters.DirectoryName;
                    try
                    {
                        folderWatcher.EnableRaisingEvents = true;

                        folderWatcher.Created += OnFolderAdded;
                        folderWatcher.Deleted += OnFolderDeleted;
                        folderWatcher.Renamed += OnFolderRenamed;
                    }
                    catch { }
                });
        }

        private void OnFolderAdded(object sender, FileSystemEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(() =>
            {
                if (!subFolders.Items.Any(f => f.Path == e.FullPath))
                {
                    subFolders.Add(new FolderTreeItemViewModel(fileSystem, folderWatcherFactory, noParallel: noParallel) { Path = e.FullPath });
                }
            });
        }

        private void OnFolderDeleted(object sender, FileSystemEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(() =>
            {
                var item = subFolders.Items.FirstOrDefault(f => f.Path == e.FullPath);

                if (item != null) subFolders.Remove(item);
            });
        }

        private void OnFolderRenamed(object sender, RenamedEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(() =>
            {
                var item = subFolders.Items.FirstOrDefault(f => f.Path == e.OldFullPath);

                if (item != null)
                {
                    subFolders.Remove(item);

                    subFolders.Add(new FolderTreeItemViewModel(fileSystem, folderWatcherFactory, noParallel) { Path = e.FullPath });
                }
            });
        }

        ~FolderTreeItemViewModel()
        {
            if (folderWatcher != null)
            {
                folderWatcher.Created -= OnFolderAdded;
                folderWatcher.Deleted -= OnFolderDeleted;
                folderWatcher.Renamed -= OnFolderRenamed;
            }

            disposableRegistration.Dispose();
        }
    }
}