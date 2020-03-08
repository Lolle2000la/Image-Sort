using ImageSort.WPF.FolderIcons;
using System;
using System.Globalization;
using System.IO;
using System.Windows.Data;

namespace ImageSort.WPF.Converters
{
    class PathToFolderIconConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value == null) throw new ArgumentNullException(nameof(value));

            if (value is string path)
            {
                if (!Directory.Exists(path)) throw new DirectoryNotFoundException();

                return ShellFileLoader.GetThumbnailFromShellForWpf(path);
            }

            throw new ArgumentException("Value should be a string.", nameof(value));
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
