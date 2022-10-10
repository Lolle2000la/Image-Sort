using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using MetadataExtractor;

namespace ImageSort.FileSystem
{
    public class FullAccessFileSystemMetadataExtractor : IMetadataExtractor
    {
        public Dictionary<string, Dictionary<string, string>> Extract(string x)
        {
            var dict = new Dictionary<string, Dictionary<string, string>>();
            var directories = ImageMetadataReader.ReadMetadata(x);
            foreach (var directory in directories)
            {
                var subDict = new Dictionary<string, string>();
                foreach (var tag in directory.Tags)
                {
                    subDict.Add(tag.Name, tag.Description);
                }
                dict.Add(directory.Name, subDict);
            }
            return dict;
        }
    }
}
