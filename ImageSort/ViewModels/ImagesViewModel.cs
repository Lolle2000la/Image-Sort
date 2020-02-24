using DynamicData;
using DynamicData.Binding;
using ImageSort.FileSystem;
using ImageSort.Helpers;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive;
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

        public ReactiveCommand<Unit, Unit> GoLeft { get; }
        public ReactiveCommand<Unit, Unit> GoRight { get; }

        public ImagesViewModel(IFileSystem fileSystem = null)
        {
            fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();

            images = new SourceList<string>();

            images.Connect()
                .Sort(SortExpressionComparer<string>.Ascending(p => p))
                .Filter(this.WhenAnyValue(x => x.SearchTerm)
                    .Select<string, Func<string, bool>>(t => p => t == null || p.Contains(t, StringComparison.OrdinalIgnoreCase)))
                .Bind(out _images)
                .Subscribe();

            this.WhenAnyValue(x => x.CurrentFolder)
                .Where(f => f != null)
                .Select(f => fileSystem.GetFiles(f)
                                      .Where(s => s.EndsWithAny(
                                          StringComparison.OrdinalIgnoreCase,
                                          ".png", ".jpg", ".gif", ".bmp", ".tiff", ".tif", ".ico")))
                .Subscribe(i=> 
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

            GoLeft = ReactiveCommand.Create(() => {
                SelectedIndex--; }, canGoLeft);

            var canGoRight = this.WhenAnyValue(x => x.SelectedIndex, x => x.Images.Count, (i, _) => i)
                .Select(i => i < Images.Count - 1);

            GoRight = ReactiveCommand.Create(() => { 
                SelectedIndex++; }, canGoRight);
        }

        public void RemoveImage(string image)
        {
            images.Remove(image);
        }

        public void InsertImage(string image)
        {
            images.Add(image);
        }

        ~ImagesViewModel()
        {
            images.Dispose();
        }
    }
}