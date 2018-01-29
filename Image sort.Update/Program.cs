using Newtonsoft.Json;
using System;
using System.Collections.Generic;
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
                    // if the version given is different, download and run the newest update
                    if (updateReg.version != Properties.Resources.version)
                        if (MessageBox.Show("Do you want to update to the newest" +
                            " version of Image sort?", "Update", MessageBoxButton.YesNo,
                            MessageBoxImage.Question) == MessageBoxResult.Yes)
                            DownloadAndRunInstaller(updateReg);
            }
        }

        /// <summary>
        /// 
        /// </summary>
        /// <returns></returns>
        public static string GetUpdateRegistry()
        {
            string json;
            using (WebClient wc = new WebClient())
            {
                try
                {
                    json = wc.DownloadString("https://raw.githubusercontent.com" +
                        "/Lolle2000la/Image-Sort/master/update-reg.json");
                }
                catch (WebException ex)
                {
                    MessageBox.Show("Server does not answer", "Warning!", MessageBoxButton.OK, MessageBoxImage.Warning);
                    json = "";
                }
            }
            return json;
        }

        public static void DownloadAndRunInstaller(UpdateRegModel updateReg)
        {
            using (WebClient wc = new WebClient())
            {
                try
                {
                    wc.DownloadFile(updateReg.url, "setup.msi");
                    System.Diagnostics.Process.Start(AppDomain.CurrentDomain.BaseDirectory
                        + @"\setup.msi");
                }
                catch (WebException ex)
                {
                    MessageBox.Show("Server does not answer", "Warning!", MessageBoxButton.OK, MessageBoxImage.Warning);
                }
            }
        }
    }
}
