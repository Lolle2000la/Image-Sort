using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text;

namespace ImageSort.ViewModels
{
    public class FolderTreeItemViewModel : ReactiveObject
    {
        private string _path;
        public string Path
        {
            get => _path;
            set => this.RaiseAndSetIfChanged(ref _path, value);
        }

        private readonly ObservableAsPropertyHelper<string> _folderName;
        public string FolderName => _folderName.Value;

        private readonly ObservableAsPropertyHelper<IEnumerable<FolderTreeItemViewModel>> _children;
        public IEnumerable<FolderTreeItemViewModel> Children => _children.Value;

        public FolderTreeItemViewModel(IFileSystem fileSystem = null, IScheduler backgroundScheduler = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();
            backgroundScheduler = backgroundScheduler ?? RxApp.TaskpoolScheduler;

            _folderName = this.WhenAnyValue(x => x.Path)
                .Select(p => 
                {
                    var path = System.IO.Path.GetFileName(p);

                    return path == "" ? p : path; // on a disk path (e.g. C:\, Path.GetFileName() returns an empty string
                })
                .ToProperty(this, x => x.FolderName);

            _children = this.WhenAnyValue(x => x.Path)
                            .Where(p => p != null)
                            .Select(p =>
                            {
                                try
                                {
                                    return fileSystem
                                        .GetSubFolders(p)
                                        .Select(folder =>
                                        {
                                            try
                                            {
                                                return new FolderTreeItemViewModel(fileSystem, backgroundScheduler) { Path = folder };
                                            }
                                            catch (UnauthorizedAccessException ex) { return null; }
                                        })
                                        .Where(f => f != null);
                                }
                                catch (UnauthorizedAccessException ex) { return null; }
                            })
                            .ObserveOn(backgroundScheduler)
                            .ToProperty(this, x => x.Children);
        }
    }
}
