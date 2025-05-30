using ImageSort.FileSystem; // Added: For IFileSystem, IRecycleBin
using System.Reactive.Concurrency; // Added: For IScheduler
using ImageSort.ViewModels;
using ReactiveUI;
using Splat;
using System.Reactive;
using Avalonia.Platform.Storage;
using Avalonia.Controls; // Added for TopLevel
using Avalonia.Controls.ApplicationLifetimes; // Added for IClassicDesktopStyleApplicationLifetime
using Application = Avalonia.Application; // Added for Application.Current
using System.Linq; // Added for .Any()

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
                interaction.SetOutput(result[0].Path.LocalPath);
            }
            else
            {
                interaction.SetOutput(null);
            }
        });
    }

    // Remove placeholder properties like Greeting and commands,
    // as they are now inherited from ImageSort.ViewModels.MainViewModel
    // e.g., public ReactiveCommand<Unit, Unit> OpenFolder { get; } is in MainViewModel
}
