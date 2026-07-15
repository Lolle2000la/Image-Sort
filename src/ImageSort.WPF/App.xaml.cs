using System;
using System.Globalization;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Threading;
using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.SettingsManagement;
using ImageSort.WPF.FileSystem;
using ImageSort.WPF.SettingsManagement;
using ImageSort.WPF.SettingsManagement.ShortCutManagement;
using ImageSort.WPF.SettingsManagement.WindowPosition;
using ReactiveUI;
using ReactiveUI.Builder;
using Splat;

#if !DO_NOT_INCLUDE_UPDATER
using System.Collections.Generic;
using AdonisUI.Controls;
using ImageSort.Localization;
using Octokit;
using ImageSort.WindowsUpdater;
using System.Linq;

#endif

namespace ImageSort.WPF;

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

        RxAppBuilder.CreateReactiveUIBuilder()
            .WithExceptionHandler(Observer.Create<Exception>(ex =>
            {
                System.Windows.Application.Current?.Dispatcher.Invoke(() =>
                {
                    System.Windows.MessageBox.Show(
                        $"ReactiveUI Pipeline Exception:\n\n{ex.Message}\n\nStack Trace:\n{ex.StackTrace}",
                        "ReactiveUI Error",
                        System.Windows.MessageBoxButton.OK,
                        System.Windows.MessageBoxImage.Error);
                });
            }))
            .WithWpf()
            .BuildApp();

        Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetEntryAssembly());
        Locator.CurrentMutable.RegisterManditoryDependencies();
        Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
        Locator.CurrentMutable.RegisterSettings(settings =>
        {
            settings.Add(new GeneralSettingsGroupViewModel());
            settings.Add(new PinnedFolderSettingsViewModel());
            settings.Add(new KeyBindingsSettingsGroupViewModel());
            settings.Add(new WindowPositionSettingsViewModel<MainWindow>());
            settings.Add(new MetadataPanelSettings());
        });
        Locator.CurrentMutable.RegisterLazySingleton(() => new SettingsViewModel());

        Startup += OnStartup;

        Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;

        DispatcherUnhandledException += UnhandledDispatcherException;
        TaskScheduler.UnobservedTaskException += TaskScheduler_UnobservedTaskException;
        AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
    }

    private void UnhandledDispatcherException(object sender, DispatcherUnhandledExceptionEventArgs e)
    {
        System.Diagnostics.Trace.WriteLine(e.Exception);

        var ex = e.Exception;
        var message = ex.Message;
        var stackTrace = ex.StackTrace;

        if (ex.InnerException != null)
        {
            message += $"\n\nINNER EXCEPTION:\n{ex.InnerException.Message}";
            stackTrace = ex.InnerException.StackTrace;
        }

        System.Windows.MessageBox.Show(
            $"Unhandled UI Exception:\n\n{message}\n\nStack Trace:\n{stackTrace}",
            "Critical UI Error",
            System.Windows.MessageBoxButton.OK,
            System.Windows.MessageBoxImage.Error);

        e.Handled = true;
    }

    private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
    {
        System.Diagnostics.Trace.WriteLine(e.ExceptionObject);

        var exception = e.ExceptionObject as Exception;
        var message = exception != null
            ? $"{exception.Message}\n\nStack Trace:\n{exception.StackTrace}"
            : e.ExceptionObject?.ToString();

        System.Windows.Application.Current?.Dispatcher.Invoke(() =>
        {
            System.Windows.MessageBox.Show(
                $"Unhandled AppDomain Exception:\n\n{message}",
                "Critical Domain Error",
                System.Windows.MessageBoxButton.OK,
                System.Windows.MessageBoxImage.Error);
        });
    }

    private void TaskScheduler_UnobservedTaskException(object sender, UnobservedTaskExceptionEventArgs e)
    {
        System.Diagnostics.Trace.WriteLine(e.Exception);

        System.Windows.Application.Current?.Dispatcher.Invoke(() =>
        {
            System.Windows.MessageBox.Show(
                $"Unobserved Task Exception:\n\n{e.Exception.InnerException?.Message ?? e.Exception.Message}",
                "Task Error",
                System.Windows.MessageBoxButton.OK,
                System.Windows.MessageBoxImage.Warning);
        });

        e.SetObserved();
    }

    // Warning is disabled, since async is used when running in release mode for automatic updates.
#pragma warning disable CS1998 // Async method lacks 'await' operators and will run synchronously
    private async void OnStartup(object sender, StartupEventArgs e)
#pragma warning restore CS1998 // Async method lacks 'await' operators and will run synchronously
    {
        var settings = Locator.Current.GetService<SettingsViewModel>();

        settings.Restore();
        
#if !DO_NOT_INCLUDE_UPDATER
        InstallerRunner.CleanUpInstaller();

        var generalSettings = Locator.Current.GetServices<SettingsGroupViewModelBase>()
            .OfType<GeneralSettingsGroupViewModel>()
            .Single();

        if (generalSettings.CheckForUpdatesOnStartup)
        {
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
        }

        {
            var ghubClientV3 = new GitHubClient(new ProductHeaderValue("Image-Sort"));
            var updateFetcherV3 = new GitHubUpdateFetcher(ghubClientV3);
            var hasStableV3Release = await updateFetcherV3.HasStableV3ReleaseAsync().ConfigureAwait(true);
            MainWindow.MediaSortV3Available = hasStableV3Release;

            if (hasStableV3Release && this.MainWindow is MainWindow mainWindow)
            {
                mainWindow.ShowMediaSortAd();
            }
        }
#endif
    }

    ~App()
    {
        Startup -= OnStartup;
    }
}