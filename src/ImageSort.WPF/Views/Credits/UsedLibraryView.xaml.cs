using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Navigation;

namespace ImageSort.WPF.Views.Credits;

/// <summary>
///     Interaction logic for UsedLibraryView.xaml
/// </summary>
public partial class UsedLibraryView : UserControl
{
    // Using a DependencyProperty as the backing store for LibraryName.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty LibraryNameProperty =
        DependencyProperty.Register("LibraryName", typeof(string), typeof(UsedLibraryView),
            new PropertyMetadata("Library Name"));

    // Using a DependencyProperty as the backing store for ProjectUrl.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty ProjectUrlProperty =
        DependencyProperty.Register("ProjectUrl", typeof(string), typeof(UsedLibraryView),
            new PropertyMetadata("https://example.com/"));

    // Using a DependencyProperty as the backing store for LicenseUrl.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty LicenseUrlProperty =
        DependencyProperty.Register("LicenseUrl", typeof(string), typeof(UsedLibraryView),
            new PropertyMetadata("https://example.com/"));

    public UsedLibraryView()
    {
        InitializeComponent();
        DataContext = this;
    }

    public string LibraryName
    {
        get => (string) GetValue(LibraryNameProperty);
        set => SetValue(LibraryNameProperty, value);
    }

    [SuppressMessage("Design", "CA1056:Uri properties should not be strings",
        Justification = "Uris are not easily bindable to literal strings in xaml.")]
    public string ProjectUrl
    {
        get => (string) GetValue(ProjectUrlProperty);
        set => SetValue(ProjectUrlProperty, value);
    }

    [SuppressMessage("Design", "CA1056:Uri properties should not be strings",
        Justification = "Uris are not easily bindable to literal strings in xaml.")]
    public string LicenseUrl
    {
        get => (string) GetValue(LicenseUrlProperty);
        set => SetValue(LicenseUrlProperty, value);
    }

    private void OnRequestNavigate(object sender, RequestNavigateEventArgs e)
    {
        Process.Start(new ProcessStartInfo
        {
            FileName = e.Uri.AbsoluteUri,
            UseShellExecute = true
        });
    }
}