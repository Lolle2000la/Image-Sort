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

        private int _selectedIndex;
        public int SelectedIndex
        {
            get => _selectedIndex;
            set => this.RaiseAndSetIfChanged(ref _selectedIndex, value);
        }

        private ObservableAsPropertyHelper<string> _selectedImage;
        public string SelectedImage => _selectedImage.Value; 

        public ImagesViewModel(IFileSystem fileSystem = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();

            _images = this.WhenAnyValue(x => x.CurrentFolder)
                .Where(f => f != null)
                .Select(f => fileSystem.GetFiles(f)
                                      .Where(s => s.EndsWithAny(
                                          StringComparison.OrdinalIgnoreCase,
                                          ".png", ".jpg", ".gif", ".bmp", ".tiff", ".tif", ".ico")))
                .ToProperty(this, x => x.Images);

            this.WhenAnyValue(x => x.Images)
                .Where(i => i != null && i.Any())
                .Take(1)
                .Subscribe(_ =>
                {
                    _selectedImage = this.WhenAnyValue(x => x.SelectedIndex)
                        .Where(i => Images.Any())
                        .Where(i => i >= 0)
                        .Select(i => Images.ElementAt(i))
                        .ToProperty(this, x => x.SelectedImage);
                });

            this.WhenAnyValue(x => x.Images)
                .Subscribe(_ => SelectedIndex = 0);
        }
    }
}