﻿using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
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

        private bool _isExpanded = false;
        public bool IsExpanded
        {
            get => _isExpanded;
            set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
        }

        private readonly ObservableAsPropertyHelper<IEnumerable<FolderTreeItemViewModel>> _children;
        public IEnumerable<FolderTreeItemViewModel> Children => _children.Value;

        public FolderTreeItemViewModel(IFileSystem fileSystem = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();

            _children = this
                .WhenAnyValue(x => x.IsExpanded)
                .Where(b => b)
                .Take(1)
                .Select(p =>
                {
                    try
                    {
                        return fileSystem
                            .GetSubFolders(_path)
                            .Select(folder => new FolderTreeItemViewModel(fileSystem) { Path = folder });
                    }
                    catch (UnauthorizedAccessException)
                    { return null; }
                })
                .ToProperty(this, x => x.Children);
        }
    }
}