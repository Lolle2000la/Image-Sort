using System.Collections.Generic;

namespace ImageSort.FileSystem
{
    public interface IMetadataExtractor
    {
        Dictionary<string, Dictionary<string, string>> Extract(string x);
    }
}