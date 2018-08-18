using System;
using System.Collections.Generic;
using System.Configuration;
using System.Data;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        [DllImport("wininet.dll")]
        public extern static bool InternetGetConnectedState(out int Description, int ReservedValue);

        /// <summary>
        /// Contains an <see cref="Process"/> object with the updater.
        /// </summary>
        public static Process updaterProcess = null;

        /// <summary>
        /// Gets internet state, returns true if connected
        /// </summary>
        /// <returns></returns>
        public static bool IsConnectedToInternet()
        {
            return InternetGetConnectedState(out int Desk, 0);
        }

        /// <summary>
        /// Overridden so that the updater can check for updates before startup
        /// </summary>
        /// <param name="e"></param>
        protected override void OnStartup(StartupEventArgs e)
        {
#if !IS_UWP
            // Run updater, if connected to internet and the updater exists.
            if (IsConnectedToInternet() && File.Exists(AppDomain.CurrentDomain.BaseDirectory
                    + @"\Image sort.Update.exe"))
                // Run the Updater before starting the app
                updaterProcess = new Process()
                {
                    StartInfo = new ProcessStartInfo()
                    {
                        FileName = AppDomain.CurrentDomain.BaseDirectory
                                    + @"\Image sort.Update.exe",
                        UseShellExecute = false,
                        RedirectStandardError = true,
                        RedirectStandardInput = true,
                        RedirectStandardOutput = true
                    }
                };
#endif

#if DEBUG && !DEBUG_LOCALIZATION
            CultureInfo ci = new CultureInfo("en");
            Thread.CurrentThread.CurrentCulture = ci;
            Thread.CurrentThread.CurrentUICulture = ci;
#endif

            // Continue normally
            base.OnStartup(e);

            // Makes sure, the dialogs look nice and native
            System.Windows.Forms.Application.EnableVisualStyles();
        }
    }
}
