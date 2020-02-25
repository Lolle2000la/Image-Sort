using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Media.Imaging;

namespace Image_sort.UI.Classes
{
    /// <summary>
    /// Contains helpers fo usage with <see cref="Bitmap"/>
    /// and <see cref="BitmapImage"/>.
    /// </summary>
    static class BitmapHelper
    {
        /// <summary>
        /// Converts the given <see cref="Bitmap"/> to an <see cref="BitmapImage"/>.
        /// </summary>
        /// <param name="bitmap">The <see cref="Bitmap"/> to be converted.</param>
        /// <returns>the converted <see cref="Bitmap"/>.</returns>
        public static BitmapImage ToBitmapImage(this Bitmap bitmap)
        {
            using (var memory = new MemoryStream())
            {
                bitmap.Save(memory, ImageFormat.Png);
                memory.Position = 0;

                var bitmapImage = new BitmapImage();
                bitmapImage.BeginInit();
                bitmapImage.StreamSource = memory;
                bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
                bitmapImage.EndInit();
                bitmapImage.Freeze();

                return bitmapImage;
            }
        }

        /// <summary>
        /// Converts the given <see cref="Bitmap"/> to an <see cref="BitmapImage"/> asynchronously.
        /// </summary>
        /// <param name="bitmap">The <see cref="Bitmap"/> to be converted.</param>
        /// <returns>the converted <see cref="Bitmap"/>.</returns>
        public async static Task<BitmapImage> ToBitmapImageAsync(this Bitmap bitmap)
        {
            return await Task.Factory.StartNew(() => ToBitmapImage(bitmap));
        }
    }
}
