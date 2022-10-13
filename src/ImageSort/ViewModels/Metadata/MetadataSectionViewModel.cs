using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels.Metadata;
public class MetadataSectionViewModel : ReactiveObject
{
    private string _title;
    public string Title
    {
        get => _title;
        set => this.RaiseAndSetIfChanged(ref _title, value);
    }

    private Dictionary<string, string> _fields;
    public Dictionary<string, string> Fields
    {
        get => _fields;
        set => this.RaiseAndSetIfChanged(ref _fields, value);
    }
    
    private ObservableAsPropertyHelper<IEnumerable<MetadataFieldViewModel>> _fieldViewModels;
    public IEnumerable<MetadataFieldViewModel> FieldViewModels => _fieldViewModels.Value;

    public MetadataSectionViewModel(MetadataFieldViewModelFactory fieldViewModelFactory)
    {
        _fieldViewModels = this.WhenAnyValue(x => x.Fields)
            .Select(f => f.Select(x => fieldViewModelFactory.Create(x.Key, x.Value)))
            .ToProperty(this, x => x.FieldViewModels);
    }
}
