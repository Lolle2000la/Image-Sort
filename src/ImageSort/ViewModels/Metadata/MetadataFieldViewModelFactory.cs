using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels.Metadata;
public class MetadataFieldViewModelFactory
{
    public MetadataFieldViewModel Create(string name, string value)
    {
        return new()
        {
            Name = name,
            Value = value
        };
    }
}
