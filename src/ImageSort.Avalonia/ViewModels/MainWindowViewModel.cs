using ImageSort.FileSystem; // Added: For IFileSystem, IRecycleBin
using System.Reactive.Concurrency; // Added: For IScheduler
using ImageSort.ViewModels;
using ReactiveUI;
using Splat;
using System;
using System.Reactive;
using Avalonia.Platform.Storage;
using System.Linq; // Added for .Any()
using MsBox.Avalonia; // For message boxes
using MsBox.Avalonia.Enums; // For message box button/icon enums
using System.Threading.Tasks; // For Task
using ImageSort.Avalonia.Views; // For InputDialog
using ImageSort.Localization; // For Text resource
using ImageSort.Avalonia.Input; // Required for AppAction
using System.Reactive.Linq; // Required for ObserveOn
using Avalonia.Controls.ApplicationLifetimes; // Required for IClassicDesktopStyleApplicationLifetime
using Avalonia.Controls; // Required for TopLevel
using Application = Avalonia.Application; // Required for Application.Current

namespace ImageSort.Avalonia.ViewModels;

// Inherit from the core MainViewModel
public partial class MainWindowViewModel : MainViewModel
{
    // Commands for Settings, Keybindings, Credits - specific to Avalonia UI or to be handled here
    public ReactiveCommand<Unit, Unit> OpenSettings { get; }
    public ReactiveCommand<Unit, Unit> OpenKeybindings { get; }
    public ReactiveCommand<Unit, Unit> OpenCredits { get; }

    // Commands for AppActions that are not already in base ViewModels
    public ReactiveCommand<Unit, Unit> FocusSearchBoxCommand { get; }
    public ReactiveCommand<Unit, Unit> ToggleMetadataPanelCommand { get; }
    // PinCurrentFolder is in FoldersViewModel
    // PinSelectedTreeFolder is in FoldersViewModel
    // UnpinSelectedTreeFolder is in FoldersViewModel
    // CreateFolderInSelectedTreeFolder is in FoldersViewModel

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

        FocusSearchBoxCommand = ReactiveCommand.Create(() => 
        {
            // This needs to interact with the UI. One way is via an Interaction.
            // Or, if the search box is a known element, perhaps a direct focus call if possible (less MVVM).
            // For now, let's define an interaction.
            RequestFocusSearchBox.Handle(Unit.Default);
        });

        ToggleMetadataPanelCommand = ReactiveCommand.Create(() =>
        {
            // This will toggle a boolean property that the Metadata panel's visibility is bound to.
            // Let's assume such a property exists or will be added, e.g., IsMetadataPanelVisible.
            // For now, let's define an interaction or a direct property change.
            // Let's add a property to ImagesViewModel for this.
            Images.IsMetadataVisible = !Images.IsMetadataVisible; 
        });

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

    // Interaction for focusing the search box
    public Interaction<Unit, Unit> RequestFocusSearchBox { get; } = new Interaction<Unit, Unit>();

    public void ExecuteAppAction(AppAction action)
    {
        switch (action)
        {
            // Image Navigation
            case AppAction.NextImage:
                Images.SelectNextImage.Execute().Subscribe();
                break;
            case AppAction.PreviousImage:
                Images.SelectPreviousImage.Execute().Subscribe();
                break;

            // Action History
            case AppAction.Undo:
                Actions.Undo.Execute().Subscribe();
                break;
            case AppAction.Redo:
                Actions.Redo.Execute().Subscribe();
                break;

            // Image Operations
            case AppAction.MoveImageToCurrentSelectedFolder: // This is the "Up Arrow" or similar action
                // This action implies the currently selected folder in the main grid/view, not the tree.
                // This maps to the "Move" command in ActionsViewModel, which uses the FoldersViewModel.CurrentFolder
                // The "CurrentFolder" in FoldersViewModel is what the image should be moved to.
                // We need to ensure Folders.SelectedFolder is set appropriately if this action means something different.
                // For now, assuming it means move to Folders.CurrentFolder (which is the active one for dropping images)
                Actions.Move.Execute().Subscribe(); // Move uses the current image from ImagesViewModel and current folder from FoldersViewModel
                break;
            case AppAction.DeleteImage:
                Images.DeleteImageCommand.Execute().Subscribe(); // Corrected from Images.DeleteImage
                break;
            case AppAction.RenameImage:
                Images.RenameImage.Execute().Subscribe();
                break;

            // Folder Tree Navigation/Selection
            case AppAction.SelectNextFolderInTree:
                Folders.SelectNextFolder.Execute().Subscribe();
                break;
            case AppAction.SelectPreviousFolderInTree:
                Folders.SelectPreviousFolder.Execute().Subscribe();
                break;
            case AppAction.ExpandSelectedTreeFolder:
                Folders.ExpandSelected.Execute().Subscribe();
                break;
            case AppAction.CollapseSelectedTreeFolderOrGoToParent:
                Folders.CollapseSelectedOrGoToParent.Execute().Subscribe();
                break;
            case AppAction.SetSelectedTreeFolderAsCurrent: // Enter on tree item
                Folders.SetSelectedFolderAsCurrentImplicitly.Execute().Subscribe(); // Or a new command if different behavior needed
                break;

            // Pinned Folder Image Move Operations
            case AppAction.MoveImageToPinnedFolder1: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[0]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder2: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[1]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder3: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[2]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder4: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[3]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder5: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[4]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder6: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[5]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder7: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[6]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder8: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[7]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder9: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[8]?.Path).Subscribe(); break;
            case AppAction.MoveImageToPinnedFolder0: Actions.MoveImageToFolder.Execute(Folders.PinnedFolders[9]?.Path).Subscribe(); break;
                // Note: Need to ensure PinnedFolders collection is accessed safely (e.g., check count or nulls)
                // This is handled by the ?.Path and the command in ActionsViewModel should handle null path if necessary.

            // UI Control/Focus
            case AppAction.FocusSearchBox:
                FocusSearchBoxCommand.Execute().Subscribe();
                break;
            case AppAction.ToggleMetadataPanel:
                ToggleMetadataPanelCommand.Execute().Subscribe();
                break;

            // Application Level
            case AppAction.OpenFolderDialog: // O key
                OpenFolder.Execute().Subscribe(); // This is in MainViewModel base
                break;
            case AppAction.PinCurrentFolder: // P key
                Folders.PinCurrentFolder.Execute().Subscribe();
                break;
            case AppAction.PinSelectedTreeFolder: // F key
                Folders.PinSelected.Execute().Subscribe(); // Corrected from PinSelectedFolder
                break;
            case AppAction.UnpinSelectedTreeFolder: // U key
                Folders.UnpinSelected.Execute().Subscribe(); // Corrected from UnpinSelectedFolder
                break;
            case AppAction.CreateFolderInSelectedTreeFolder: // C key
                Folders.CreateFolderUnderSelected.Execute().Subscribe();
                break;
            
            default:
                // Log or handle unknown action
                break;
        }
    }
}
