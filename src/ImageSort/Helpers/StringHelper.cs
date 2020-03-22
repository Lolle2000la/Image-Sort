using System;

namespace ImageSort.Helpers
{
    internal static class StringHelper
    {
        public static bool EndsWithAny(
            this string @string,
            StringComparison comparisonType,
            params string[] atEnd)
        {
            foreach (var end in atEnd)
            {
                if (@string.EndsWith(end, comparisonType)) return true;
            }

            return false;
        }
    }
}