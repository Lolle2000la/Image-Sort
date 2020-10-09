using System;
using System.Globalization;
using System.Reactive.Concurrency;
using System.Reflection;
using System.Threading;
using System.Windows;
using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.SettingsManagement;
using ImageSort.WPF.FileSystem;
using ImageSort.WPF.SettingsManagement;
using ImageSort.WPF.SettingsManagement.ShortCutManagement;
using ImageSort.WPF.SettingsManagement.WindowPosition;
using ReactiveUI;
using Splat;

#if !DO_NOT_INCLUDE_UPDATER
using System.Collections.Generic;
using AdonisUI.Controls;
using ImageSort.Localization;
using Octokit;
using ImageSort.WindowsUpdater;
using System.Linq;

#endif

namespace ImageSort.WPF
{
    /// <summary>
    ///     Interaction logic for App.xaml
    /// </summary>
    public partial class App : System.Windows.Application
    {
        public App()
        {
#if DEBUG && !DEBUG_LOCALIZATION
            Thread.CurrentThread.CurrentCulture = new CultureInfo("en");
            Thread.CurrentThread.CurrentUICulture = new CultureInfo("en");
#endif

            Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetEntryAssembly());
            Locator.CurrentMutable.RegisterManditoryDependencies();
            Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
            Locator.CurrentMutable.RegisterSettings(settings =>
            {
                settings.Add(new GeneralSettingsGroupViewModel());
                settings.Add(new PinnedFolderSettingsViewModel());
                settings.Add(new KeyBindingsSettingsGroupViewModel());
                settings.Add(new WindowPositionSettingsViewModel<MainWindow>());
            });
            Locator.CurrentMutable.RegisterLazySingleton(() => new SettingsViewModel());

            Startup += OnStartup;

            Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;
        }

        private async void OnStartup(object sender, StartupEventArgs e)
        {
            RxApp.MainThreadScheduler.Schedule(async () =>
            {
                var settings = Locator.Current.GetService<SettingsViewModel>();

                settings.Restore();
            });

#if !DO_NOT_INCLUDE_UPDATER
            InstallerRunner.CleanUpInstaller();

            var generalSettings = Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
                .OfType<GeneralSettingsGroupViewModel>()
                .Single();

            if (!generalSettings.CheckForUpdatesOnStartup) return;

            var ghubClient = new GitHubClient(new ProductHeaderValue("Image-Sort"));
            var updateFetcher = new GitHubUpdateFetcher(ghubClient);
            (var success, var release) = 
                await updateFetcher.TryGetLatestReleaseAsync(generalSettings.InstallPrereleaseBuilds).ConfigureAwait(true);

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
                    Icon = AdonisUI.Controls.MessageBoxImage.Question
                };

                if (AdonisUI.Controls.MessageBox.Show(messageBox) == AdonisUI.Controls.MessageBoxResult.Yes && updateFetcher.TryGetInstallerFromRelease(release, out var installerAsset))
                {
                    var installer = await updateFetcher.GetStreamFromAssetAsync(installerAsset).ConfigureAwait(false);

                    InstallerRunner.RunAsync(installer).ConfigureAwait(false);
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