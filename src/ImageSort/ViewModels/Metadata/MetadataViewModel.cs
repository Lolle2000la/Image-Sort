using ImageSort.FileSystem;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive;
using System.Reactive.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels.Metadata;

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

    private ObservableAsPropertyHelper<IEnumerable<MetadataSectionViewModel>> _sectionViewModels;
    public IEnumerable<MetadataSectionViewModel> SectionViewModels => _sectionViewModels.Value;

    private bool _isExpanded = true;
    public bool IsExpanded
    {
        get => _isExpanded;
        set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
    }

    public ReactiveCommand<Unit, Unit> ToggleIsExpanded { get; }

    public MetadataViewModel(IMetadataExtractor extractor, IFileSystem fileSystem, MetadataSectionViewModelFactory metadataSectionFactory)
    {
        this.extractor = extractor;
        this.fileSystem = fileSystem;

        _metadata = this.WhenAnyValue(x => x.ImagePath)
            .Select(ExtractSafely)
            .ToProperty(this, x => x.Metadata);

        _sectionViewModels = this.WhenAnyValue(x => x.Metadata)
            .Where(x => x.Type == MetadataResultType.Success)
            .Select(m => m.Metadata.OrderBy(x => x.Key))
            .Select(m => m.Select(d => metadataSectionFactory.Create(d.Key, d.Value)))
            .ToProperty(this, x => x.SectionViewModels);

        ToggleIsExpanded = ReactiveCommand.Create(() =>
        {
            IsExpanded = !IsExpanded;
        });
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
        // since we don't want an exception to take down the application and instead pass it on, we catch all of them here
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
