using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Text;
using System.Threading.Tasks;
using System.Windows;

namespace Image_sort.Update
{
    class Program
    {

        /// <summary>
        /// Main method of the app, everything is in here
        /// </summary>
        /// <param name="args"></param>
        static void Main(string[] args)
        {
            // Loads update registry from GitHub
            string json = GetUpdateRegistry();
            
            // Checks if something was given back
            if (json != "")
            {
                // Serializes the UpdateRegistry from json
                UpdateRegModel updateReg = JsonConvert.DeserializeObject<UpdateRegModel>(json);
                if (updateReg != null)
                    // if the version given is different, download and run the newest! update
                    if (updateReg.version != Properties.Resources.version)
                        if (MessageBox.Show("Do you want to update to the newest" +
                            " version of Image sort?", "Update", MessageBoxButton.YesNo,
                            MessageBoxImage.Question) == MessageBoxResult.Yes)
                            DownloadAndRunInstaller(updateReg);
            }
        }

        /// <summary>
        /// Downloads the registry from the GitHub server
        /// </summary>
        /// <returns>Returns it a as a string in JSON form</returns>
        public static string GetUpdateRegistry()
        {
            // Used to keep the string
            string json;

            // Downloads the file or notifies the server if it wasn't possible 
            using (WebClient wc = new WebClient())
            {
                try
                {
                    json = wc.DownloadString(Properties.Resources.UpdateRegistryUrl);
                }
                catch (WebException)
                {
                    MessageBox.Show("Server does not answer", "Warning!", MessageBoxButton.OK, MessageBoxImage.Warning);
                    json = "";
                }
            }
            return json;
        }

        /// <summary>
        /// Downloads and runs the installer from the newest version specified in the given registry
        /// </summary>
        /// <param name="updateReg"></param>
        public static void DownloadAndRunInstaller(UpdateRegModel updateReg)
        {
            // Downloads the installer
            using (WebClient wc = new WebClient())
            {
                // Downloads the installer from the given URL as setup
                try
                {
                    if(updateReg.url != null)
                    {
                        // Download the installer
                        wc.DownloadFile(updateReg.url, "setup.msi");
                        // Set the target path for it
                        string target = AppDomain.CurrentDomain.BaseDirectory + @"\setup.msi";
                        // Run it and wait for it to exit
                        System.Diagnostics.Process.Start(target).WaitForExit();
                        // Delete the installer
                        File.Delete(target);
                    }
                    else
                    {
                        MessageBox.Show("Update server did not return an url to the installer!", "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                    }
                }
                // If something goes wrong, show the user that it didn't
                catch (WebException)
                {
                    MessageBox.Show("Server does not answer", "Warning!", MessageBoxButton.OK, MessageBoxImage.Warning);
                }
            }
        }
    }
}
