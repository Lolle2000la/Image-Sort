using ImageSort.Localization;
using LazyCache;
using LazyCache.Providers;
using Microsoft.Extensions.Caching.Memory;
using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace ImageSort.WPF.FileSystem
{
    internal static class ImageLoading
    {
        static IAppCache cache = new CachingService(
            new MemoryCacheProvider(
                new MemoryCache(
                    new MemoryCacheOptions()
                    {
                        SizeLimit = 20, // limit the maximum number of 
                    })));

        static MemoryCacheEntryOptions options = new MemoryCacheEntryOptions()
        {
            Size = 1, // the same unit must be used, as MemoryCache itself knows no units. Here, 1 equals 1 element.
        };

        public static ImageSource GetImageFromPath(string path)
        {
            if (path == null) return null;

            return cache.GetOrAdd<ImageSource>(path, () =>
            {
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
            }, options);
        }
    }
}