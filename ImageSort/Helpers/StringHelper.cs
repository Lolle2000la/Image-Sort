using System;

namespace ImageSort.Helpers
{
    static class StringHelper
    {
        public static bool EndsWithAny(
            this string @string,
            StringComparison comparisonType,
            params string[] atEnd)
        {
            var endsWithIt = false;

            foreach(var end in atEnd)
            {
                if (@string.EndsWith(end, comparisonType)) endsWithIt = true;
            }

            return endsWithIt;
        }
    }
}
