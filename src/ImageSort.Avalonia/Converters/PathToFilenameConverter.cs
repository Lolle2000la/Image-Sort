using Avalonia.Data.Converters;
using System;
using System.Globalization;
using System.IO;

namespace ImageSort.Avalonia.Converters
{
    public class PathToFilenameConverter : IValueConverter
    {
        public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        {
            if (value is string path && !string.IsNullOrEmpty(path))
            {
                try
                {
                    return Path.GetFileName(path);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Error getting filename from {path}: {ex.Message}");
                    return path; // Fallback to full path on error
                }
            }
            return string.Empty;
        }

        public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
