﻿using System;
using System.Globalization;
using System.IO;
using System.Windows.Data;

namespace ImageSort.WPF.Converters;

[ValueConversion(typeof(string), typeof(string))]
internal class PathToFilenameConverter : IValueConverter
{
    public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
    {
        if (value == null) return "";

        if (value is string path) return Path.GetFileName(path);

        return "";
    }

    public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}