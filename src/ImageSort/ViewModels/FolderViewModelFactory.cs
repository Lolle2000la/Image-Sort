using ImageSort.FileSystem;
using System;
using System.IO;
using System.Reactive.Concurrency;

namespace ImageSort.ViewModels
{
    public class FolderViewModelFactory
    {
        private readonly IFileSystem fileSystem;
        private readonly Func<FileSystemWatcher> folderWatcherFactory;
        private readonly IScheduler backgroundScheduler;

        public FolderViewModelFactory(IFileSystem fileSystem, Func<FileSystemWatcher> folderWatcherFactory, IScheduler backgroundScheduler)
        {
            this.fileSystem = fileSystem;
            this.folderWatcherFactory = folderWatcherFactory;
            this.backgroundScheduler = backgroundScheduler;
        }

        public FolderViewModel GetFor(string path)
        {
            return new FolderViewModel(fileSystem, folderWatcherFactory, backgroundScheduler)
            {
                Path = path
            };
        }
    }
}
