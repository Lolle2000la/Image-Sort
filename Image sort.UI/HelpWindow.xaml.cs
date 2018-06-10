using MahApps.Metro.Controls;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Text;
using System.Threading.Tasks;
using System.Timers;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Shapes;
using System.Windows.Threading;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for HelpWindow.xaml
    /// </summary>
    public partial class HelpWindow : MetroWindow
    {
        /// <summary>
        /// Indicates whether the <see cref="HelpWindow"/> should actually close AND unload, or just hide,
        /// when the user closes it.
        /// </summary>
        public bool DoNotClose { get; set; } = true;

        public HelpWindow()
        {
            InitializeComponent();
        }

        /// <summary>
        /// Used to prevent closing from unloading the window.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void HelpWindow_Closing(object sender, System.ComponentModel.CancelEventArgs e)
        {
            if (DoNotClose)
            {
                // taken from https://balajiramesh.wordpress.com/2008/07/24/hide-a-window-instead-of-closing-it-in-wpf/
                //Hide Window
                Application.Current.Dispatcher.BeginInvoke(DispatcherPriority.Background, (DispatcherOperationCallback)delegate (object o)
                {
                    Hide();
                    return null;
                }, null);
                //Do not close application
                e.Cancel = true;
            }
        }

        /// <summary>
        /// Opens the links.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void HelpViewer_RequestNavigate(object sender, System.Windows.Navigation.RequestNavigateEventArgs e)
        {
            Process.Start(e.Uri.OriginalString);
        }

        private void HyperlinkCommand_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            // empty, needed for launchable links.
        }

        private async void MetroWindow_Loaded(object sender, RoutedEventArgs e)
        {
            // Inserts the HELP.md into the markdown. This way, we don't need to touch the xaml in order
            // to edit the help text.
            await Task.Run(() =>
            {
                try
                {
                    // Downloads the help file from github.
                    using (WebClient wc = new WebClient())
                    {
                        // download markdown from github
                        string downloadedMarkdown = wc.DownloadString("https://raw.githubusercontent.com/Lolle2000la/Image-Sort/master/HELP.md?raw=true");

                        // Load the markdown into the HelpViewer
                        Dispatcher.Invoke(() => HelpViewer.Markdown = downloadedMarkdown);
                        // Write the markdown into the local HELP.md file to keep that up-to-date
                        File.WriteAllText(AppDomain.CurrentDomain.BaseDirectory + "\\HELP.md", $"Note: this is an offline " +
                            $"version. It has been last updated at {DateTime.Now.ToLongDateString()}. {Environment.NewLine}" +
                            $"---" +
                            $"{Environment.NewLine}{Environment.NewLine}" +
                            $"{downloadedMarkdown}");
                    }
                }
                catch (WebException)
                {
                    // if not possible, fall back to offline file.
                    if (File.Exists(AppDomain.CurrentDomain.BaseDirectory + "\\HELP.md"))
                        Dispatcher.Invoke(() => HelpViewer.Markdown = File.ReadAllText(AppDomain.CurrentDomain.BaseDirectory + "\\HELP.md"));
                    // Show error text if even that fails.
                    else
                        Dispatcher.Invoke(() => HelpViewer.Markdown = "# Could not load the help file. \r\n Make sure it exists.");

                }
            });
        }
    }
}
