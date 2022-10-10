using ImageSort.FileSystem;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels
{
    public class MetadataViewModel : ReactiveObject
    {
        private readonly IMetadataExtractor extractor;
        private readonly IFileSystem fileSystem;

        private string _imagePath;
        public string ImagePath
        {
            get => _imagePath;
            set => this.RaiseAndSetIfChanged(ref _imagePath, value);
        }

        private ObservableAsPropertyHelper<Dictionary<string, Dictionary<string, string>>> _metadata;
        public Dictionary<string, Dictionary<string, string>> Metadata => _metadata.Value;

        public MetadataViewModel(IMetadataExtractor extractor, IFileSystem fileSystem)
        {
            this.extractor = extractor;
            this.fileSystem = fileSystem;

            _metadata = this.WhenAnyValue(x => x.ImagePath)
                .Select(x => fileSystem.FileExists(x) ? extractor.Extract(x) : null)
                .ToProperty(this, x => x.Metadata);
        }
    }
}
