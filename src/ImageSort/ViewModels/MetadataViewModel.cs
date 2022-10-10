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

        private ObservableAsPropertyHelper<MetadataResult> _metadata;
        public MetadataResult Metadata => _metadata.Value;

        public MetadataViewModel(IMetadataExtractor extractor, IFileSystem fileSystem)
        {
            this.extractor = extractor;
            this.fileSystem = fileSystem;

            _metadata = this.WhenAnyValue(x => x.ImagePath)
                .Select(ExtractSafely)
                .ToProperty(this, x => x.Metadata);
        }

        private MetadataResult ExtractSafely(string path)
        {
            try
            {
                if (fileSystem.FileExists(path))
                {
                    return new()
                    {
                        Type = MetadataResultType.Success,
                        Metadata = extractor.Extract(path)
                    };
                }
                else
                {
                    return new MetadataResult()
                    {
                        Type = MetadataResultType.FileDoesNotExist
                    };
                }
            }
#pragma warning disable CA1031 // Do not catch general exception types
            catch (Exception ex)
            {
                return new MetadataResult()
                {
                    Type = MetadataResultType.UnexpectedError,
                    Exception = ex
                };
            }
#pragma warning restore CA1031 // Do not catch general exception types
        }
    }

    public record MetadataResult
    {
        public MetadataResultType Type { get; init; }
        public Dictionary<string, Dictionary<string, string>> Metadata { get; init; }
        public Exception Exception { get; init; }
    }

    public enum MetadataResultType
    {
        Success,
        FileDoesNotExist,
        UnexpectedError
    }
}
