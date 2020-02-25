using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Shapes;
using Image_sort.UI.LocalResources.AppResources;
using MahApps.Metro.Controls;
using Microsoft.Toolkit.Wpf.UI.Controls;

namespace Image_sort.UI.Windows
{
    /// <summary>
    /// Interaction logic for FeedbackWIndo.xaml
    /// </summary>
    public partial class FeedbackWindow : MetroWindow
    {
        public string FormUrl { get; } = "https://docs.google.com/forms/d/e/1FAIpQLSeRLmo5uw0ZTqrgFAYqVE5Wyfthh_BeSCCG19FYmhADwiSRcw/viewform";

        public FeedbackWindow()
        {
            InitializeComponent();
        }

        private void OnOpenInBrowserClicked(object sender, RoutedEventArgs e)
        {
            Process.Start(FormUrl);

            Close();
        }

        private void OnNavigationCompleted(object sender, Microsoft.Toolkit.Win32.UI.Controls.Interop.WinRT.WebViewControlNavigationCompletedEventArgs e)
        {
            if (e.Uri.OriginalString.EndsWith("/formResponse"))
                Close();
        }
    }
}
