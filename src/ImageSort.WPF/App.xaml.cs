using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.WPF.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Linq;
using System.Reflection;
using Application = System.Windows.Application;
using ImageSort.WPF.SettingsManagement;
using ImageSort.SettingsManagement;
using System.Reactive.Concurrency;

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
            System.Threading.Thread.CurrentThread.CurrentCulture = new System.Globalization.CultureInfo("en");
            System.Threading.Thread.CurrentThread.CurrentUICulture = new System.Globalization.CultureInfo("en");
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

            Startup += OnStartup;

            Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;

            Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetEntryAssembly());
            Locator.CurrentMutable.RegisterManditoryDependencies();
            Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
            Locator.CurrentMutable.RegisterSettings(settings =>
            {
                settings.Add(new GeneralSettingsGroupViewModel());
            });
            Locator.CurrentMutable.RegisterLazySingleton(() => new SettingsViewModel());
        }

        private async void OnStartup(object sender, System.Windows.StartupEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(async () =>
            {
                var settings = Locator.Current.GetService<SettingsViewModel>();

                await settings.RestoreAsync().ConfigureAwait(true);
            });

#if !DO_NOT_INCLUDE_UPDATER
            InstallerRunner.CleanUpInstaller();

            if (!Settings.Default.ShouldCheckForUpdates) return;

            var ghubClient = new GitHubClient(new ProductHeaderValue("Image-Sort"));
            var updateFetcher = new GitHubUpdateFetcher(ghubClient);
            (var success, var release) = await updateFetcher.TryGetLatestReleaseAsync(Settings.Default.UpdateToPrereleaseBuilds);

            if (success)
            {
                var messageBox = new MessageBoxModel
                {
                    Caption = Text.UpdateAvailablePromptTitle,
                    Text = Text.UpdateAvailablePromptText.Replace("{TagName}", release.TagName ?? "NO TAG INFORMATION AVAILABLE", StringComparison.OrdinalIgnoreCase),
                    Buttons = new[]
                    {
                        MessageBoxButtons.Yes(Text.Update),
                        MessageBoxButtons.No(Text.DoNotUpdate)
                    },
                    Icon = MessageBoxImage.Question
                };

                if (MessageBox.Show(messageBox) == MessageBoxResult.Yes && updateFetcher.TryGetInstallerFromRelease(release, out var installerAsset))
                {
                    var installer = await updateFetcher.GetStreamFromAssetAsync(installerAsset);

                    InstallerRunner.RunAsync(installer);
                }
            }
#endif
        }

        ~App()
        {
            Startup -= OnStartup;
        }
    }
}