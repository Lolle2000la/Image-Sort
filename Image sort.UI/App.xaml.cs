using System;
using System.Collections.Generic;
using System.Configuration;
using System.Data;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        /// <summary>
        /// Overridden so that the updater can check for updates before startup
        /// </summary>
        /// <param name="e"></param>
        protected override void OnStartup(StartupEventArgs e)
        {
            // Run the Updater before starting the app
            System.Diagnostics.Process.Start(AppDomain.CurrentDomain.BaseDirectory 
                + @"\Image sort.Update.exe").WaitForExit();

            // Continue normally
            base.OnStartup(e);
        }
    }
}
