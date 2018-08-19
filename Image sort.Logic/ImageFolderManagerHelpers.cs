using System;
using System.IO;

namespace Image_sort.Logic
{
    /// <summary>
    /// Contains helpers used in the <see cref="ImageFolderManager"/> class.
    /// </summary>
    public static class ImageFolderManagerHelpers
    {
        /// <summary>
        /// Determines whether a <see cref="string"/> ends with one of the <see cref="string"/>s given in an 
        /// <see cref="Array"/> of <see cref="string"/>s.
        /// </summary>
        /// <param name="source"></param>
        /// <param name="endings"></param>
        /// <returns></returns>
        public static bool EndsWithEither(this string source, string[] endings)
        {
            bool endsWithOne = false;
            foreach (string ending in endings)
            {
                if (source.EndsWith(ending))
                {
                    endsWithOne = true;
                    break;
                }
            }
            return endsWithOne;
        }

        /// <summary>
        /// Takes a Path as a <see cref="string"/> and gives it back with a number
        /// ("image.jpg" -> "image(i).jpg) of i
        /// </summary>
        /// <param name="original">The original string that should be used</param>
        /// <param name="i">The number which should get inserted</param>
        /// <returns></returns>
        public static string GetPathWithNumber(string original, int i)
        {
            /* First get the directory, in which the original path lives in, 
             * then add the file name without extension at the end of it,
             * add the number between the (),
             * and finally add the extension back at it again. */
            return Path.GetDirectoryName(original) + @"\" +
                            Path.GetFileNameWithoutExtension(original) +
                            $"({i.ToString()})" +
                            Path.GetExtension(original);
        }
    }
}
