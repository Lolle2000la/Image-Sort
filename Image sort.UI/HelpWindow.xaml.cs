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
        public HelpWindow()
        {
            InitializeComponent();

            // Inserts the HELP.md into the markdown. This way, we don't need to touch the xaml in order
            // to edit the help text.
#if !DEBUG_HELP
            HelpViewer.Markdown = File.ReadAllText(AppDomain.CurrentDomain.BaseDirectory + "\\HELP.md");
#else
            HelpViewer.Markdown = File.ReadAllText(AppDomain.CurrentDomain.BaseDirectory + "..\\..\\HELP.md");

            Timer reloadTimer = new Timer();
            reloadTimer.Interval = 1000;
            reloadTimer.Enabled = true;
            reloadTimer.Elapsed += (s, e) =>
            {
                string markdown = File.ReadAllText(AppDomain.CurrentDomain.BaseDirectory + "..\\..\\HELP.md");
                Func<bool> compare = () => { return HelpViewer.Markdown != markdown; };
                if (Dispatcher.Invoke<bool>(compare))
                {
                    Dispatcher.Invoke(() => HelpViewer.Markdown = markdown);
                }
            };
            reloadTimer.Start();
#endif
        }

        /// <summary>
        /// Used to prevent closing from unloading the window.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void HelpWindow_Closing(object sender, System.ComponentModel.CancelEventArgs e)
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
    }
}
