using ImageSort.SettingsManagement;
using ImageSort.WPF.FileSystem;
using ImageSort.WPF.SettingsManagement;
using Splat;
using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Windows.Data;
using System.Windows.Media.Imaging;

namespace ImageSort.WPF.Converters
{
    [ValueConversion(typeof(string), typeof(BitmapImage))]
    internal class PathToBitmapImageConverter : IValueConverter
    {
        public int? LoadWidth { get; set; } = null;
        public bool ForGifThumbnails { get; set; } = false;
        
        private GeneralSettingsGroupViewModel generalSettings = Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
                .Select(s => s as GeneralSettingsGroupViewModel)
                .First(s => s != null);

        [SuppressMessage("Design", "CA1031:Do not catch general exception types",
            Justification = "The app should not crash just because some exception happened")]
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null) return null;
            if (ForGifThumbnails && (!generalSettings.AnimateGifThumbnails || !generalSettings.AnimateGifs)) return null; // prevent gifs from loading when disabled.

            if (value is string path)
            {
                if (ForGifThumbnails && Path.GetExtension(path).ToUpperInvariant() != ".GIF") return null;
                return ImageLoading.GetImageFromPath(path);
            }

            return null;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}