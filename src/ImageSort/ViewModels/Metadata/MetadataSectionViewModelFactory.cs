using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.ViewModels.Metadata;
public class MetadataSectionViewModelFactory
{
    public MetadataSectionViewModel Create(string title, Dictionary<string, string> fields)
    {
        return new()
        {
            Title = title,
            Fields = fields
        };
    }
}
