using Image_sort.Communication;
using Image_sort.Update.GitHub;
using Newtonsoft.Json;
using Octokit;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Security.Permissions;
using System.Security.Principal;
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
#if (!DEBUG || DEBUG_UPDATER)
            // If the updater is already open once, don't open another instance/
            // close this one right after start.
            if (Process.GetProcessesByName("Image sort.Update").Count() > 1)
                Environment.Exit(0);

            // Makes sure, the dialogs look nice and native
            System.Windows.Forms.Application.EnableVisualStyles();

            // Loads update registry from GitHub
            //string json = GetUpdateRegistry();

            // Use the GitHub Api to get the latest release
            var ghub = new GitHubClient(new ProductHeaderValue("Image-sort"));
            string latestVersion = "";
            Release release = null;
            try
            {
                release = ghub.Repository.Release.GetLatest("Lolle2000la", "Image-Sort").Result;
                latestVersion = release.TagName;
            }
            catch (RateLimitExceededException ex)
            {
                // Notify the accessing app of the reached rate limit and when it's going to reset.
                Console.WriteLine(UpdaterConstants.RateLimitReached);
                Console.WriteLine(UpdaterConstants.ResetsOnTime);
                Console.WriteLine(ex.Reset.UtcDateTime);
            }
            // when that doesn't work, tell the main app.
            catch (ApiException ex)
            {
                Console.WriteLine(UpdaterConstants.Error);
                Console.WriteLine(ex.StatusCode);
            }

            //// Get the latest release of image sort on GitHub.
            //GithubRelease latestRelease = GetLatestReleaseInfo();

            // Checks if something was given back
            if (release != null)
            {
                // Serializes the UpdateRegistry from json
                //UpdateRegModel updateReg = JsonConvert.DeserializeObject<UpdateRegModel>(json);

                // makes sure the latest release isn't a prerelease.
                if (!release.Prerelease)
                {
                    // if the version given is different, download and run the newest update
                    if (IsVersionNew(release))
                    {
                        InitUpdating(release);
                    }
                    // if thats not the case subscribe to the latest release, to check for updates.
                    else
                    {
                        // create a new object for handling releases.
                        var observableReleases = new Octokit.Reactive.ObservableReleasesClient(ghub);
                        // keeps track of whether the running instances should be looked after.
                        bool checkForRunningInstances = true;
                        // subscribe to the latest release
                        observableReleases.GetLatest("Lolle2000la", "Image-Sort").Subscribe((newRelease) =>
                        {
                            // check if the latest release is not a prerelease and a new version.
                            if (!newRelease.Prerelease && IsVersionNew(newRelease))
                            {
                                checkForRunningInstances = false;
                                InitUpdating(newRelease);
                            }
                        });
                        // check the whole time if any instance of Image sort is still running.
                        while (checkForRunningInstances)
                        {
                            if (Process.GetProcessesByName("Image sort.UI").Length == 0)
                            {
                                // if not end the loop.
                                checkForRunningInstances = false;
                            }

                            Task.Delay(2000);
                        }
                    }
                }
            }
#endif
        }

        /// <summary>
        /// Get if the given <see cref="Release"/> contains a new version.
        /// </summary>
        /// <param name="release">The release to be checked.</param>
        /// <returns>Whether the latest release is new.</returns>
        private static bool IsVersionNew(Release release)
        {
            // Get the version of Image sort.UI
            Version version = System.Reflection.Assembly.LoadFile($"{AppDomain.CurrentDomain.BaseDirectory}\\Image sort.UI.exe")
                .GetName().Version;
            return release.TagName.Substring(1) != $"{version.Major}.{version.Minor}.{version.Build}";
        }

        /// <summary>
        /// Inits the updating process, aks the accessing app for consent first.
        /// </summary>
        /// <param name="release">The release with which updating should get initialised.</param>
        private static void InitUpdating(Release release)
        {
            // If the process isn't elevated, ask if update
            if (!IsElevated)
            {
                // asks the parent process for user consent
                Console.WriteLine(UpdaterConstants.UserConsent);
                // asks if consent is given (yes the true : false is for performance optimization)
                bool consentGiven = (Console.ReadLine() == UpdaterConstants.Positive) ? true : false;

                // if consent was given, then elavate the process and start it.
                if (consentGiven)
                {
                    // Elevate process
                    ProcessStartInfo info = new ProcessStartInfo(AppDomain.CurrentDomain.BaseDirectory +
                        @"Image sort.Update.exe") {
                        UseShellExecute = true,
                        Verb = "runas"
                    };
                    Process.Start(info);
                }
                else
                {
                    Environment.Exit(0);
                }
            }
            // If it is, download and run the installer
            else
            {
                // set the url depending on if one of them is set
                string url = "";

                // Get the asset, that is an installer.
                foreach (var asset in release.Assets)
                {
                    if (asset.Name.EndsWith(".msi"))
                    {
                        url = asset.BrowserDownloadUrl;
                    }
                }

                // Check if the url even exists.
                if (url != "")
                    // Download and install the installer
                    DownloadAndRunInstaller(url);
            }
        }

        /// <summary>
        /// Get the latest release info from GitHub.
        /// </summary>
        /// <returns></returns>
        private static GithubRelease GetLatestReleaseInfo()
        {
            using (WebClient wc = new WebClient())
            {
                // GitHub requires an user-agent, but it's not important what its value is.
                wc.Headers.Add(HttpRequestHeader.UserAgent, "agent");

                // Download the JSON from GitHub and encode it to UTF8
                string json = Encoding.UTF8.GetString(wc.DownloadData(Properties.Resources.LatestReleaseUrl));

                // Deserialize the JSON to an Object and return it.
                return JsonConvert.DeserializeObject<GithubRelease>(json);
            }
        }


        #region Methods
        /// <summary>
        /// Downloads the registry from the GitHub server
        /// </summary>
        /// <returns>Returns it a as a string in JSON form</returns>
        [Obsolete("Image sort just downloads the latest release from GitHub in the future. \r\n" +
            "Please use GetLatestReleaseInfo()", true)]
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
                    System.Windows.Forms.MessageBox.Show(Resources.AppResources.ServerNotAnswering, Resources.AppResources.Warning,
                        System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);

                    json = "";
                }
            }
            return json;
        }

        /// <summary>
        /// Downloads and runs the installer from the newest version specified in the given registry
        /// </summary>
        /// <param name="updateReg"></param>
        public static void DownloadAndRunInstaller(string url)
        {
            // Try killing the main app
            try
            {
                // For every process with the name "Image sort.UI.exe", kill it
                foreach (Process proc in Process.GetProcessesByName("Image sort.UI"))
                {
                    // kill process.
                    proc.Kill();
                }
            }
            // If an error occurs
            catch (Exception ex)
            {
                // Notify the calling app of the error.
                Console.WriteLine(UpdaterConstants.Error);
                Console.WriteLine(ex.Message);
            }

            // Downloads the installer
            using (WebClient wc = new WebClient())
            {
                // Downloads the installer from the given URL as setup
                try
                {
                    // Make sure everything is cleaned up.
                    DeleteSetup();

                    if (url != null)
                    {
                        // Makes sure everything is cleaned up.
                        DeleteSetup();

                        // Set the target path for it in User %AppData%
                        string target = Path.Combine(Environment.GetFolderPath
                            (Environment.SpecialFolder.ApplicationData), @"\setup.msi");
                        // Download the installer
                        wc.DownloadFile(url, target);
                        // Run it and wait for it to exit
                        Process.Start(target, "/passive");

                        // Save installer location
                        LastInstallerPath = target;
                    }
                    else
                    {
                        System.Windows.Forms.MessageBox.Show(Resources.AppResources.UpdateServerDidNotRespondWithUrl,
                            Resources.AppResources.Error, System.Windows.Forms.MessageBoxButtons.OK,
                            System.Windows.Forms.MessageBoxIcon.Error);
                        // GitHub now opens show the user the updates
                        Process.Start("https://github.com/Lolle2000la/Image-Sort/releases");
                    }
                }
                // If something goes wrong, show the user that it didn't
                catch (WebException)
                {
                    System.Windows.Forms.MessageBox.Show(Resources.AppResources.ServerNotAnswering,
                        Resources.AppResources.Warning, System.Windows.Forms.MessageBoxButtons.OK,
                        System.Windows.Forms.MessageBoxIcon.Error);
                    // GitHub now opens show the user the updates
                    Process.Start("https://github.com/Lolle2000la/Image-Sort/releases");
                }
                catch (Exception)
                {
                    System.Windows.Forms.MessageBox.Show(Resources.AppResources.CouldNotInstall,
                        Resources.AppResources.Warning, System.Windows.Forms.MessageBoxButtons.OK,
                        System.Windows.Forms.MessageBoxIcon.Error);
                    // GitHub now opens show the user the updates
                    Process.Start("https://github.com/Lolle2000la/Image-Sort/releases");
                }
            }
        }

        /// <summary>
        /// Stores and gives back the path to the last installer
        /// </summary>
        public static string LastInstallerPath
        {
            get
            {
                return Properties.Settings.Default.LastPathToInstaller;
            }
            set
            {
                Properties.Settings.Default.LastPathToInstaller = value;
                Properties.Settings.Default.Save();
            }
        }

        /// <summary>
        /// Checks if the process is elevated beforehand
        /// </summary>
        public static bool IsElevated
        {
            get
            {
                bool isElevated;
                using (WindowsIdentity identity = WindowsIdentity.GetCurrent())
                {
                    WindowsPrincipal principal = new WindowsPrincipal(identity);
                    isElevated = principal.IsInRole(WindowsBuiltInRole.Administrator);
                }
                return isElevated;
            }

        }

        /// <summary>
        /// Looks if there is an installer left, that should be deleted
        /// </summary>
        public static void DeleteSetup()
        {
            // If there already is an installer left, delete it.
            if (File.Exists(LastInstallerPath))
            {
                // Delete it
                File.Delete(LastInstallerPath);
            }
        }
        #endregion
    }
}
