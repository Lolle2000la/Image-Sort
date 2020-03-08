﻿using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.Localization;
using ImageSort.WPF.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using Application = System.Windows.Application;
#if !DO_NOT_INCLUDE_UPDATER
using Octokit;
using ImageSort.WindowsUpdater;
#endif

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
#if DEBUG && !DEBUG_LOCALIZATION
            Thread.CurrentThread.CurrentCulture = new CultureInfo("en");
            Thread.CurrentThread.CurrentUICulture = new CultureInfo("en");
#endif

            var assembly = Assembly.GetAssembly(typeof(App));
            var gitVersionInformationType = assembly.GetType("GitVersionInformation");
            var versionTag = (string)gitVersionInformationType.GetFields().First(f => f.Name == "SemVer").GetValue(null);

            if (Settings.Default.OldVersion != versionTag)
            {
                Settings.Default.Upgrade();
                Settings.Default.OldVersion = versionTag;
                Settings.Default.Save();
            }

#if !DO_NOT_INCLUDE_UPDATER
            Startup += OnStartup;
#endif

            Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;

            Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetEntryAssembly());
            Locator.CurrentMutable.RegisterManditoryDependencies();
            Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
        }

#if !DO_NOT_INCLUDE_UPDATER
        private async void OnStartup(object sender, StartupEventArgs e)
        {
            InstallerRunner.CleanUpInstaller();

            if (!Settings.Default.ShouldCheckForUpdates) return;

            var ghubClient = new GitHubClient(new ProductHeaderValue("Image-Sort"));
            var updateFetcher = new GitHubUpdateFetcher(ghubClient);
            (var success, var release) = await updateFetcher.TryGetLatestReleaseAsync(Settings.Default.UpdateToPrereleaseBuilds);

            if (success && MessageBox.Show(Text.UpdateAvailablePromptText.Replace("{TagName}", release.TagName, StringComparison.OrdinalIgnoreCase),
                    Text.UpdateAvailablePromptTitle, MessageBoxButton.YesNo, MessageBoxImage.Question) == MessageBoxResult.Yes)
            {
                if (updateFetcher.TryGetInstallerFromRelease(release, out var installerAsset))
                {
                    var installer = await updateFetcher.GetStreamFromAssetAsync(installerAsset);

                    InstallerRunner.RunAsync(installer);
                }
            }
        }

        ~App()
        {
            Startup -= OnStartup;
        }
#endif
    }
}