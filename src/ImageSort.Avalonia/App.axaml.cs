using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core;
using Avalonia.Data.Core.Plugins;
using System.Linq;
using Avalonia.Markup.Xaml;
using ImageSort.Avalonia.ViewModels;
using ImageSort.Avalonia.Views;
using System;
using System.Globalization;
using System.Reactive.Concurrency;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.SettingsManagement;
using ReactiveUI;
using Splat;
using ImageSort.Avalonia.SettingsManagement; // For the new SettingsHelper and Restore extension
using ImageSort.ViewModels; // Added for core ViewModels
using System.IO; // Required for FileSystemWatcher
using ImageSort.Avalonia.FileSystem; // Added for TemporaryRecycleBin

#if !DO_NOT_INCLUDE_UPDATER
#endif

namespace ImageSort.Avalonia;

public partial class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
#if DEBUG && !DEBUG_LOCALIZATION
        Thread.CurrentThread.CurrentCulture = new CultureInfo("en");
        Thread.CurrentThread.CurrentUICulture = new CultureInfo("en");
#endif

        Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetExecutingAssembly());
        Locator.CurrentMutable.RegisterManditoryDependencies();
        Locator.CurrentMutable.RegisterSettings(settings =>
        {
            settings.Add(new GeneralSettingsGroupViewModel());
            settings.Add(new PinnedFolderSettingsViewModel());
            settings.Add(new KeyBindingsSettingsGroupViewModel());
            settings.Add(new MetadataPanelSettings());
        });
        Locator.CurrentMutable.RegisterLazySingleton(() => new SettingsViewModel());
        Locator.CurrentMutable.Register<IRecycleBin>(() => new TemporaryRecycleBin()); // Register TemporaryRecycleBin

        Environment.CurrentDirectory = AppDomain.CurrentDomain.BaseDirectory;

        TaskScheduler.UnobservedTaskException += TaskScheduler_UnobservedTaskException;
        AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DisableAvaloniaDataAnnotationValidation();
            var fileSystem = Locator.Current.GetService<IFileSystem>();
            if (fileSystem == null)
            {
                throw new InvalidOperationException("IFileSystem service not registered.");
            }

            var recycleBin = Locator.Current.GetService<IRecycleBin>();
            if (recycleBin == null)
            {
                throw new InvalidOperationException("IRecycleBin service not registered.");
            }

            var backgroundScheduler = RxApp.TaskpoolScheduler;
            var mainThreadScheduler = RxApp.MainThreadScheduler;

            var foldersViewModel = new FoldersViewModel(fileSystem, backgroundScheduler);
            // Correctly instantiate ImagesViewModel with its actual constructor signature
            var imagesViewModel = new ImagesViewModel(fileSystem, null); // Pass fileSystem and null for the optional folderWatcherFactory
            // Pass dependencies directly to ActionsViewModel constructor
            var actionsViewModel = new ActionsViewModel(imagesViewModel, foldersViewModel, fileSystem);

            var mainWindowViewModel = new MainWindowViewModel(
                foldersViewModel,
                imagesViewModel,
                actionsViewModel,
                fileSystem,
                recycleBin,
                backgroundScheduler);

            desktop.MainWindow = new MainWindow
            {
                DataContext = mainWindowViewModel, // Set DataContext
                ViewModel = mainWindowViewModel  // Explicitly set the ViewModel property
            };

            OnAppStartup();
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void DisableAvaloniaDataAnnotationValidation()
    {
        var dataValidationPluginsToRemove =
            BindingPlugins.DataValidators.OfType<DataAnnotationsValidationPlugin>().ToArray();

        foreach (var plugin in dataValidationPluginsToRemove)
        {
            BindingPlugins.DataValidators.Remove(plugin);
        }
    }

    private void CurrentDomain_UnhandledException(object? sender, UnhandledExceptionEventArgs e)
    {
        System.Diagnostics.Trace.WriteLine(e.ExceptionObject);
    }

    private void TaskScheduler_UnobservedTaskException(object? sender, UnobservedTaskExceptionEventArgs e)
    {
        System.Diagnostics.Trace.WriteLine(e.Exception);
        e.SetObserved();
    }

#pragma warning disable CS1998
    private async void OnAppStartup()
#pragma warning restore CS1998
    {
        var settings = Locator.Current.GetService<SettingsViewModel>();
        var mainWindowVm = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.MainWindow?.DataContext as MainWindowViewModel;

        if (settings != null)
        {
            settings.Restore(); // This should set CurrentFolder (if saved) and PinnedFolder paths
        }

        if (mainWindowVm != null && settings != null)
        {
            var pinnedFolderSettings = settings.GetGroup<PinnedFolderSettingsViewModel>();
            if (pinnedFolderSettings != null && pinnedFolderSettings.PinnedFolders != null)
            {
                mainWindowVm.Folders.AddPinnedFoldersFromPaths(pinnedFolderSettings.PinnedFolders);
            }

            // After settings are restored and pinned folders loaded, 
            // if CurrentFolder has been set (e.g., by settings restoration),
            // force it to re-notify. This might help ensure DisplayedFolderItems updates correctly.
            if (mainWindowVm.Folders.CurrentFolder != null)
            {
                var tempCurrentFolder = mainWindowVm.Folders.CurrentFolder;
                // Temporarily setting to null and back to the original instance 
                // to ensure property change notifications are raised.
                mainWindowVm.Folders.CurrentFolder = null; 
                mainWindowVm.Folders.CurrentFolder = tempCurrentFolder;
            }
        }

#if !DO_NOT_INCLUDE_UPDATER
#endif
    }
}