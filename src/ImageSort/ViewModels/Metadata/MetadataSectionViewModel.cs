using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
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
}
