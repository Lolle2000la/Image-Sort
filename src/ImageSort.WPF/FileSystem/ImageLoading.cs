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

namespace ImageSort.WPF.FileSystem;

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
                Rotation rotation = GetImageOrientation(path);

                bitmapImage.BeginInit();
                bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
                bitmapImage.UriSource = new Uri(path);
                bitmapImage.Rotation = rotation;
                bitmapImage.EndInit();

                if (bitmapImage.Width <= 0 || bitmapImage.Height <= 0)
                    throw new BadImageFormatException($"Image {Path.GetFileName(path)} has invalid dimensions.", path);

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

    // Required for some images to be displayed in their correct orientation. See https://github.com/Lolle2000la/Image-Sort/issues/445
    // Solution taken from StackOverflow user Lâm Quang Minh (https://stackoverflow.com/a/63627972/7147000)
    private static Rotation GetImageOrientation(string path)
    {
        const string _orientationQuery = "System.Photo.Orientation";
        Rotation rotation = Rotation.Rotate0;
        using (FileStream fileStream = new FileStream(path, FileMode.Open, FileAccess.Read))
        {
            BitmapFrame bitmapFrame = BitmapFrame.Create(fileStream, BitmapCreateOptions.DelayCreation, BitmapCacheOption.None);
            BitmapMetadata bitmapMetadata = bitmapFrame.Metadata as BitmapMetadata;

            if ((bitmapMetadata != null) && (bitmapMetadata.ContainsQuery(_orientationQuery)))
            {
                object o = bitmapMetadata.GetQuery(_orientationQuery);

                if (o != null)
                {
                    switch ((ushort)o)
                    {
                        case 6:
                            {
                                rotation = Rotation.Rotate90;
                            }
                            break;
                        case 3:
                            {
                                rotation = Rotation.Rotate180;
                            }
                            break;
                        case 8:
                            {
                                rotation = Rotation.Rotate270;
                            }
                            break;
                    }
                }
            }
        }

        return rotation;
    }
}