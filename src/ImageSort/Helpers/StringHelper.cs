using System;
using System.Linq;

namespace ImageSort.Helpers
{
    internal static class StringHelper
    {
        public static bool EndsWithAny(
            this string @string,
            StringComparison comparisonType,
            params string[] atEnd)
        {
            return atEnd.Any(end => @string.EndsWith(end, comparisonType));
        }
    }
}