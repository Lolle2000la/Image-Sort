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
        /// string to the base help file inside the application folder.
        /// </summary>
        private readonly string baseHelpPath = AppDomain.CurrentDomain.BaseDirectory + "\\HELP.md";
        /// <summary>
        /// Path to the cached help file from the internet.
        /// </summary>
        private readonly string cachedHelpPath = System.IO.Path.GetTempPath() + "\\Image_sort\\HELP.md";
        /// <summary>
        /// URL to the HELP.md file online.
        /// </summary>
        private readonly string webHelpUrl = LocalResources.Help.HelpUrls.HelpUrl;
        /// <summary>
        /// Path to the local temp directory of the app.
        /// </summary>
        private readonly string localTempDirectory = System.IO.Path.GetTempPath() + "\\Image_sort";

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
                        // download markdown from github and convert the line endings.
                        string downloadedMarkdown = wc.DownloadString(webHelpUrl).Replace("\n", Environment.NewLine);

                        // if something was returned, then 
                        if (downloadedMarkdown != "")
                        {
                            // Load the markdown into the HelpViewer
                            Dispatcher.Invoke(() => HelpViewer.Markdown = downloadedMarkdown);

                            // 
                            if (!Directory.Exists(localTempDirectory))
                                Directory.CreateDirectory(localTempDirectory);

                            // Write the markdown into the local HELP.md file to keep that up-to-date
                            File.WriteAllText(cachedHelpPath, $"Note: this is an offline " +
                                $"version. It has been last updated on {DateTime.Now.ToLongDateString()}. {Environment.NewLine}" +
                                $"---" +
                                $"{Environment.NewLine}{Environment.NewLine}" +
                                $"{downloadedMarkdown}");
                        }
                        else
                        {
                            throw new WebException();
                        }
                    }
                }
                catch (WebException)
                {
                    // Load the cached help file, if it exists.
                    if (Directory.Exists(localTempDirectory) && File.Exists(cachedHelpPath))
                        Dispatcher.Invoke(() => HelpViewer.Markdown = File.ReadAllText(cachedHelpPath));
                    // if not possible, fall back to offline file.
                    else if (File.Exists(baseHelpPath))
                        Dispatcher.Invoke(() => HelpViewer.Markdown = File.ReadAllText(baseHelpPath));
                    // Show error text if even that fails.
                    else
                        Dispatcher.Invoke(() => HelpViewer.Markdown = "# Could not load the help file. \r\n Make sure it exists.");

                }
            });
        }
    }
}
