using AdonisUI.Controls;
using ImageSort.Localization;
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using System.Windows.Data;
using System.Windows.Media.Imaging;

namespace ImageSort.WPF.Converters
{
    [ValueConversion(typeof(string), typeof(BitmapImage))]
    class PathToBitmapImageConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null) throw new ArgumentNullException(nameof(value));

            if (value is string path)
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
                    MessageBox.Show(Text.CouldNotLoadImageErrorText
                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                        .Replace("{FileName}", Path.GetFileName(path), StringComparison.OrdinalIgnoreCase), Text.Error);
                }

                return null;
            }

            throw new ArgumentException("Value should be a string.", nameof(value));
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
