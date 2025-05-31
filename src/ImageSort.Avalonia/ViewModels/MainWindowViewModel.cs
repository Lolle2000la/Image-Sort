using ImageSort.FileSystem; // Added: For IFileSystem, IRecycleBin
using System.Reactive.Concurrency; // Added: For IScheduler
using ImageSort.ViewModels;
using ReactiveUI;
using Splat;
using System;
using System.Reactive;
using Avalonia.Platform.Storage;
using Avalonia.Controls; // Added for TopLevel
using Avalonia.Controls.ApplicationLifetimes; // Added for IClassicDesktopStyleApplicationLifetime
using Application = Avalonia.Application; // Added for Application.Current
using System.Linq; // Added for .Any()
using MsBox.Avalonia; // For message boxes
using MsBox.Avalonia.Enums; // For message box button/icon enums
using System.Threading.Tasks; // For Task
using ImageSort.Avalonia.Views; // For InputDialog
using ImageSort.Localization; // For Text resource

namespace ImageSort.Avalonia.ViewModels;

// Inherit from the core MainViewModel
public partial class MainWindowViewModel : MainViewModel
{
    // Commands for Settings, Keybindings, Credits - specific to Avalonia UI or to be handled here
    public ReactiveCommand<Unit, Unit> OpenSettings { get; }
    public ReactiveCommand<Unit, Unit> OpenKeybindings { get; }
    public ReactiveCommand<Unit, Unit> OpenCredits { get; }

    public MainWindowViewModel(FoldersViewModel foldersViewModel, ImagesViewModel imagesViewModel, ActionsViewModel actionsViewModel, 
                                 IFileSystem fileSystem, IRecycleBin recycleBin, IScheduler backgroundScheduler)
        : base(foldersViewModel, imagesViewModel, actionsViewModel, fileSystem, recycleBin, backgroundScheduler)
    {
        // The base constructor will initialize Actions, Folders, and Images ViewModels.
        // Any Avalonia-specific initialization for MainWindowViewModel can go here.

        // Initialize Avalonia-specific commands
        OpenSettings = ReactiveCommand.Create(() => { /* TODO: Implement Avalonia Open Settings Logic */ });
        OpenKeybindings = ReactiveCommand.Create(() => { /* TODO: Implement Avalonia Open Keybindings Logic */ });
        OpenCredits = ReactiveCommand.Create(() => { /* TODO: Implement Avalonia Open Credits Logic */ });

        // The OpenFolder command in MainViewModel uses PickFolder interaction.
        // We need to register a handler for that interaction here.
        PickFolder.RegisterHandler(async interaction =>
        {
            var topLevel = TopLevel.GetTopLevel(null); // Ideally, pass a view reference or use a service to get TopLevel
            if (topLevel == null)
            {
                // Attempt to get TopLevel from the Application.Current.ApplicationLifetime if it's an IClassicDesktopStyleApplicationLifetime
                if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktopLifetime)
                {
                    topLevel = desktopLifetime.MainWindow;
                }

                if (topLevel == null)
                {
                    interaction.SetOutput(null); // Cannot get TopLevel, so can't show picker
                    return;
                }
            }

            var result = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = "Select Folder",
                AllowMultiple = false
            });

            if (result.Any())
            {
                var uri = result[0].Path;
                string path = uri.IsAbsoluteUri ? uri.AbsolutePath : uri.OriginalString;
                
                if (Uri.TryCreate(path, UriKind.Absolute, out var fileUri) && fileUri.IsFile)
                {
                    path = fileUri.LocalPath;
                }
                else if (path.StartsWith("/") && !path.StartsWith("//") && path.Length > 1 && path[1] == ':') // C:/ -> /C:/
                {
                    path = path.Substring(1);
                }
                // For Unix root, path might already be correct as "/"
                interaction.SetOutput(path);
            }
            else
            {
                interaction.SetOutput(null);
            }
        });

        // Handler for the FoldersViewModel.SelectFolder interaction (used by Pin command)
        this.Folders.SelectFolder.RegisterHandler(async interaction =>
        {
            var topLevel = TopLevel.GetTopLevel(null); 
            if (topLevel == null)
            {
                if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktopLifetime)
                {
                    topLevel = desktopLifetime.MainWindow;
                }

                if (topLevel == null)
                {
                    interaction.SetOutput(null); 
                    return;
                }
            }

            var result = await topLevel.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = "Select Folder to Pin",
                AllowMultiple = false
            });

            if (result.Any())
            {
                var uri = result[0].Path;
                string path = uri.IsAbsoluteUri ? uri.AbsolutePath : uri.OriginalString;

                if (Uri.TryCreate(path, UriKind.Absolute, out var fileUri) && fileUri.IsFile)
                {
                    path = fileUri.LocalPath;
                }
                else if (path.StartsWith("/") && !path.StartsWith("//") && path.Length > 1 && path[1] == ':')
                {
                    path = path.Substring(1);
                }
                interaction.SetOutput(path);
            }
            else
            {
                interaction.SetOutput(null);
            }
        });

        // Handler for ImagesViewModel.PromptForNewFileName
        this.Images.PromptForNewFileName.RegisterHandler(async interaction =>
        {
            var mainWindow = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.MainWindow;
            if (mainWindow == null)
            {
                interaction.SetOutput(null);
                return;
            }

            var dialog = new InputDialog // Assuming we'll create this view
            {
                Title = "Rename File",
                // We can pass the current name as a default or placeholder if needed
            };

            var result = await dialog.ShowDialog<string>(mainWindow);

            interaction.SetOutput(result);
        });

        // Handler for ImagesViewModel.NotifyUserOfError
        this.Images.NotifyUserOfError.RegisterHandler(async interaction =>
        {
            var message = interaction.Input;
            var box = MessageBoxManager.GetMessageBoxStandard("Error", message, ButtonEnum.Ok, Icon.Error);
            
            var mainWindow = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.MainWindow;
            if (mainWindow != null)
            {
                await box.ShowWindowDialogAsync(mainWindow);
            }
            else
            {
                await box.ShowAsync(); // Show as a standalone window if main window not found
            }

            interaction.SetOutput(Unit.Default);
        });

        // Handler for ActionsViewModel.NotifyUserOfError
        this.Actions.NotifyUserOfError.RegisterHandler(async interaction =>
        {
            var message = interaction.Input;
            var box = MessageBoxManager.GetMessageBoxStandard("Error", message, ButtonEnum.Ok, Icon.Error);

            var mainWindow = (Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime)?.MainWindow;
            if (mainWindow != null)
            {
                await box.ShowWindowDialogAsync(mainWindow);
            }
            else
            {
                await box.ShowAsync(); // Show as a standalone window if main window not found
            }

            interaction.SetOutput(Unit.Default);
        });
    }
}
