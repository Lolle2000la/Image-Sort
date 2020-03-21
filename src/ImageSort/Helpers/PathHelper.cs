using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace ImageSort.Helpers
{
    public static class PathHelper
    {
        public static bool PathEquals(this string path1, string path2) =>
            Path.GetFullPath(path1).Equals(Path.GetFullPath(path2), StringComparison.OrdinalIgnoreCase);
    }
}
