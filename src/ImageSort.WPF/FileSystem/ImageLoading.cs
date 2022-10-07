using ImageSort.Localization;
using System;
using System.Globalization;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace ImageSort.WPF.FileSystem
{
    internal static class ImageLoading
    {
        public static ImageSource GetImageFromPath(string path)
        {
            if (path == null) return null;

            try
            {
                var bitmapImage = new BitmapImage();

                bitmapImage.BeginInit();
                bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
                bitmapImage.UriSource = new Uri(path);
                bitmapImage.EndInit();

                return bitmapImage;
            }
            catch (Exception ex)
            {
                var textDrawing = new GeometryDrawing
                {
                    Geometry = new GeometryGroup
                    {
                        Children = new GeometryCollection(new[]
                        {
                            new FormattedText(Text.CouldNotLoadImageErrorText
                                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                                        .Replace("{FileName}", Path.GetFileName(path),
                                            StringComparison.OrdinalIgnoreCase),
                                    CultureInfo.CurrentCulture,
                                    FlowDirection.LeftToRight,
                                    new Typeface("Segoe UI"),
                                    16,
                                    Brushes.Black, 1)
                                .BuildGeometry(new Point(8, 8))
                        })
                    },
                    Brush = Brushes.Black,
                    Pen = new Pen(Brushes.White, 0.5)
                };

                return new DrawingImage(textDrawing);
            }
        }
    }
}