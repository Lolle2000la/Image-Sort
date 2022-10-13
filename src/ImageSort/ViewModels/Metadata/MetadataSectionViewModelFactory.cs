using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels.Metadata;
public class MetadataSectionViewModelFactory
{
    private readonly MetadataFieldViewModelFactory fieldViewModelFactory;

    public MetadataSectionViewModelFactory(MetadataFieldViewModelFactory fieldViewModelFactory)
    {
        this.fieldViewModelFactory = fieldViewModelFactory;
    }

    public MetadataSectionViewModel Create(string title, Dictionary<string, string> fields)
    {
        return new(fieldViewModelFactory)
        {
            Title = title,
            Fields = fields
        };
    }
}
