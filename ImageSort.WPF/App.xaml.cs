using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.Localization;
using ImageSort.WindowsUpdater;
using ImageSort.WPF.FileSystem;
using Octokit;
using ReactiveUI;
using Splat;
using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using System.Windows;
using Application = System.Windows.Application;

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            var assembly = Assembly.GetAssembly(typeof(App));
            var gitVersionInformationType = assembly.GetType("GitVersionInformation");
            var versionTag = (string)gitVersionInformationType.GetFields().First(f => f.Name == "SemVer").GetValue(null);

            if (Settings.Default.OldVersion != versionTag)
            {
                Settings.Default.Upgrade();
                Settings.Default.OldVersion = versionTag;
                Settings.Default.Save();
            }

            Startup += OnStartup;

            Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;

            Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetEntryAssembly());
            Locator.CurrentMutable.RegisterManditoryDependencies();
            Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
        }

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
    }
}
