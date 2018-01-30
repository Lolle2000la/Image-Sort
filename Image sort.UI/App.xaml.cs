﻿using System;
using System.Collections.Generic;
using System.Configuration;
using System.Data;
using System.Linq;
using System.Runtime.InteropServices;
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
        /// Gets internet state, returns true if connected
        /// </summary>
        /// <returns></returns>
        public static bool IsConnectedToInternet()
        {
            int Desk;
            return InternetGetConnectedState(out Desk, 0);
        }

        /// <summary>
        /// Overridden so that the updater can check for updates before startup
        /// </summary>
        /// <param name="e"></param>
        protected override void OnStartup(StartupEventArgs e)
        {
            // Run updater, if connected to internet
            if(IsConnectedToInternet())
                // Run the Updater before starting the app
                System.Diagnostics.Process.Start(AppDomain.CurrentDomain.BaseDirectory 
                    + @"\Image sort.Update.exe").WaitForExit();

            // Continue normally
            base.OnStartup(e);
        }
    }
}
