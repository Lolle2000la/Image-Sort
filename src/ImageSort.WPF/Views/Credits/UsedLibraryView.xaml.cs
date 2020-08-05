using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Navigation;

namespace ImageSort.WPF.Views.Credits
{
    /// <summary>
    /// Interaction logic for UsedLibraryView.xaml
    /// </summary>
    public partial class UsedLibraryView : UserControl
    {
        public UsedLibraryView()
        {
            InitializeComponent();
            DataContext = this;
        }

        public string LibraryName
        {
            get { return (string)GetValue(LibraryNameProperty); }
            set { SetValue(LibraryNameProperty, value); }
        }

        // Using a DependencyProperty as the backing store for LibraryName.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty LibraryNameProperty =
            DependencyProperty.Register("LibraryName", typeof(string), typeof(UsedLibraryView), new PropertyMetadata("Library Name"));

        [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1056:Uri properties should not be strings", Justification = "Uris are not easily bindable to literal strings in xaml.")]
        public string ProjectUrl
        {
            get { return (string)GetValue(ProjectUrlProperty); }
            set { SetValue(ProjectUrlProperty, value); }
        }

        // Using a DependencyProperty as the backing store for ProjectUrl.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty ProjectUrlProperty =
            DependencyProperty.Register("ProjectUrl", typeof(string), typeof(UsedLibraryView), new PropertyMetadata("https://example.com/"));

        [System.Diagnostics.CodeAnalysis.SuppressMessage("Design", "CA1056:Uri properties should not be strings", Justification = "Uris are not easily bindable to literal strings in xaml.")]
        public string LicenseUrl
        {
            get { return (string)GetValue(LicenseUrlProperty); }
            set { SetValue(LicenseUrlProperty, value); }
        }

        // Using a DependencyProperty as the backing store for LicenseUrl.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty LicenseUrlProperty =
            DependencyProperty.Register("LicenseUrl", typeof(string), typeof(UsedLibraryView), new PropertyMetadata("https://example.com/"));

        private void OnRequestNavigate(object sender, RequestNavigateEventArgs e)
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = e.Uri.AbsoluteUri,
                UseShellExecute = true
            });
        }
    }
}
