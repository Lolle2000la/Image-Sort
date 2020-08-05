using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Security.Policy;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

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



        public Uri ProjectUrl
        {
            get { return (Uri)GetValue(ProjectUrlProperty); }
            set { SetValue(ProjectUrlProperty, value); }
        }

        // Using a DependencyProperty as the backing store for ProjectUrl.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty ProjectUrlProperty =
            DependencyProperty.Register("ProjectUrl", typeof(Uri), typeof(UsedLibraryView), new PropertyMetadata("https://example.com/"));



        public Uri LicenseUrl
        {
            get { return (Uri)GetValue(LicenseUrlProperty); }
            set { SetValue(LicenseUrlProperty, value); }
        }

        // Using a DependencyProperty as the backing store for LicenseUrl.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty LicenseUrlProperty =
            DependencyProperty.Register("LicenseUrl", typeof(Uri), typeof(UsedLibraryView), new PropertyMetadata("https://example.com/"));

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
