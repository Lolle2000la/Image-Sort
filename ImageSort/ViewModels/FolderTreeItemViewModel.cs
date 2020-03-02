using DynamicData;
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

namespace ImageSort.ViewModels
{
    public class FolderTreeItemViewModel : ReactiveObject
    {
        private readonly CompositeDisposable disposableRegistration = new CompositeDisposable();
        private readonly IFileSystem fileSystem;
        private readonly IScheduler backgroundScheduler;
        private readonly Func<FileSystemWatcher> folderWatcherFactory;
        private readonly FileSystemWatcher folderWatcher;

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

        public FolderTreeItemViewModel(IFileSystem fileSystem = null, IScheduler backgroundScheduler = null, Func<FileSystemWatcher> folderWatcherFactory = null)
        {
            this.fileSystem = fileSystem ??= Locator.Current.GetService<IFileSystem>();
            this.backgroundScheduler = backgroundScheduler ??= RxApp.TaskpoolScheduler;
            this.folderWatcherFactory = folderWatcherFactory ??= () => Locator.Current.GetService<FileSystemWatcher>();
            folderWatcher = folderWatcherFactory();
            folderWatcher?.DisposeWith(disposableRegistration);

            subFolders = new SourceList<FolderTreeItemViewModel>();
            subFolders.Connect()
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
                .ObserveOn(backgroundScheduler)
                .Where(p => p != null)
                .Subscribe(p =>
                {
                    try
                    {
                        var _subFolders = fileSystem
                            .GetSubFolders(p);

                        if (_subFolders != null)
                        {
                            foreach (var folder in _subFolders.Where(f => f != null))
                            {
                                try
                                {
                                    subFolders.Add(new FolderTreeItemViewModel(fileSystem, backgroundScheduler) { Path = folder });
                                }
                                catch (UnauthorizedAccessException) { }
                            }
                        }
                    }
                    catch (UnauthorizedAccessException) { }
                })
                .DisposeWith(disposableRegistration);

            CreateFolder = ReactiveCommand.Create<string, Unit>(name =>
            {
                var newFolderPath = System.IO.Path.Combine(Path, name);

                if (Children.Select(f => f.Path).Any(s => s == newFolderPath)) return Unit.Default;

                fileSystem.CreateFolder(newFolderPath);

                subFolders.Add(new FolderTreeItemViewModel(fileSystem, backgroundScheduler) { Path = newFolderPath });

                return Unit.Default;
            });

            this.WhenAnyValue(x => x.Path)
                .Where(p => !string.IsNullOrEmpty(p))
                .Where(_ => folderWatcher != null)
                .Subscribe(p =>
                {
                    folderWatcher.Path = p;
                    folderWatcher.IncludeSubdirectories = false;
                    folderWatcher.EnableRaisingEvents = true;

                    folderWatcher.Created += OnFolderAdded;
                    folderWatcher.Deleted += OnFolderDeleted;
                });
        }

        private void OnFolderAdded(object sender, FileSystemEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(() => 
                subFolders.Add(new FolderTreeItemViewModel(fileSystem, backgroundScheduler, folderWatcherFactory) { Path = e.FullPath }));
        }

        private void OnFolderDeleted(object sender, FileSystemEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(() => 
            {
                var item = subFolders.Items.FirstOrDefault(f => f.Path == e.FullPath);

                if (item != null) subFolders.Remove(item);
            });
        }

        ~FolderTreeItemViewModel()
        {
            folderWatcher.Created -= OnFolderAdded;
            folderWatcher.Deleted -= OnFolderDeleted;

            disposableRegistration.Dispose();
        }
    }
}
