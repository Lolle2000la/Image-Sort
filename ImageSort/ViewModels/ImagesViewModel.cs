using ImageSort.FileSystem;
using ImageSort.Helpers;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Linq;

namespace ImageSort.ViewModels
{
    public class ImagesViewModel : ReactiveObject
    {
        private string _currentPath;
        public string CurrentFolder
        {
            get => _currentPath;
            set => this.RaiseAndSetIfChanged(ref _currentPath, value);
        }

        private readonly ObservableAsPropertyHelper<IEnumerable<string>> _images;
        public IEnumerable<string> Images => _images.Value;

        private string _currentImage;
        public string CurrentImage
        {
            get => _currentImage;
            set => this.RaiseAndSetIfChanged(ref _currentImage, value);
        }

        public ImagesViewModel(IFileSystem fileSystem = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();

            _images = this.WhenAnyValue(x => x.CurrentFolder)
                .Select(f => fileSystem.GetFiles(f)
                                      .Where(s => s.EndsWithAny(
                                          StringComparison.OrdinalIgnoreCase,
                                          ".png", ".jpg", ".gif", ".bmp", ".tiff", ".tif", ".ico")))
                .ToProperty(this, x => x.Images);
        }
    }
}