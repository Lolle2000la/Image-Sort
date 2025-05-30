using Avalonia;
using Avalonia.Controls;
using Avalonia.ReactiveUI; // Required for ReactiveWindow
using ImageSort.Avalonia.ViewModels; // Assuming this will be created or already exists
using ImageSort.ViewModels; // This is from the core ImageSort project
using ReactiveUI;
using Splat;
using System;
using System.Reactive.Disposables;
// using System.Windows.Forms; // Not available in Avalonia, find alternatives if needed for dialogs
// using System.Windows.Input; // Avalonia has its own input system (Avalonia.Input)

namespace ImageSort.Avalonia.Views;

public partial class MainWindow : ReactiveWindow<MainWindowViewModel> // Changed to ReactiveWindow<MainWindowViewModel>
{
    // private bool interceptReservedKeys = true; // Will need Avalonia equivalent for key handling

    public MainWindow()
    {
        InitializeComponent();
#if DEBUG
        this.AttachDevTools(); // Standard Avalonia practice. Requires `using Avalonia;`
#endif
        // ViewModel is set in App.axaml.cs for Avalonia MVVM template
        // ViewModel = new MainViewModel
        // {
        //     Folders = new FoldersViewModel
        //     {
        //         CurrentFolder = new FolderTreeItemViewModel
        //         {
        //             Path = Environment.GetCommandLineArgs().ElementAtOrDefault(1) ??
        //                    Environment.GetFolderPath(Environment.SpecialFolder.MyPictures)
        //         }
        //     },
        //     Images = new ImagesViewModel(),
        //     Actions = new ActionsViewModel()
        // };

        // var settings = Locator.Current.GetService<SettingsViewModel>(); // Now SettingsViewModel should be resolved

        // Closed += async (o, e) => await settings.SaveAsync().ConfigureAwait(false); // Avalonia uses WindowClosed event
        // if (settings != null) // ensure settings is not null before using
        // {
        //     this.WindowClosed += async (o, e) => await settings.SaveAsync().ConfigureAwait(false);
        // }


        this.WhenActivated(disposableRegistration =>
        {
            // Restore window state - needs Avalonia specific implementation
            // this.RestoreWindowState(); 

            // Ensure window state is saved when closing - needs Avalonia specific implementation
            // Closing += (o, e) => this.SaveWindowState();

            // ViewModel bindings will be set up here using Avalonia's binding system
            // Example (assuming Folders, Images, Actions are controls in MainWindow.axaml with Name properties):
            // this.Bind(ViewModel, vm => vm.Folders, view => view.Folders.ViewModel).DisposeWith(disposableRegistration);
            // this.Bind(ViewModel, vm => vm.Images, view => view.Images.ViewModel).DisposeWith(disposableRegistration);
            // this.OneWayBind(ViewModel, vm => vm.Actions, view => view.Actions.ViewModel).DisposeWith(disposableRegistration);

            // Command bindings
            // this.BindCommand(ViewModel, vm => vm.OpenFolder, view => view.OpenFolderButton).DisposeWith(disposableRegistration);
            // this.BindCommand(ViewModel, vm => vm.OpenCurrentlySelectedFolder, view => view.OpenSelectedFolderButton).DisposeWith(disposableRegistration);
            // this.BindCommand(ViewModel, vm => vm.MoveImageToFolder, view => view.MoveButton).DisposeWith(disposableRegistration);
            // this.BindCommand(ViewModel, vm => vm.DeleteImage, view => view.DeleteButton).DisposeWith(disposableRegistration);

            // Event handlers for settings, keybindings, credits buttons
            // var openSettingsButton = this.FindControl<Button>("OpenSettingsButton");
            // if (openSettingsButton != null) openSettingsButton.Click += OnOpenSettingsClicked;

            // var openKeybindingsButton = this.FindControl<Button>("OpenKeybindingsButton");
            // if (openKeybindingsButton != null) openKeybindingsButton.Click += OnOpenKeybindingsClicked;

            // var creditsButton = this.FindControl<Button>("CreditsButton");
            // if (creditsButton != null) creditsButton.Click += OnCreditsClicked;

            // KeyDown event for handling shortcuts - needs Avalonia specific implementation
            // this.KeyDown += MainWindow_KeyDown;
        });
    }

    // Placeholder methods for event handlers - actual implementation will depend on Avalonia dialogs/navigation
    // private void OnOpenSettingsClicked(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    // {
        // var settingsWindow = new SettingsWindowView(); // Assuming an Avalonia SettingsWindowView
        // settingsWindow.ShowDialog(this);
    // }

    // private void OnOpenKeybindingsClicked(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    // {
        // var keybindingsWindow = new KeybindingsWindowView(); // Assuming an Avalonia KeybindingsWindowView
        // keybindingsWindow.ShowDialog(this);
    // }

    // private void OnCreditsClicked(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    // {
        // var creditsWindow = new CreditsWindowView(); // Assuming an Avalonia CreditsWindowView
        // creditsWindow.ShowDialog(this);
    // }

    // Key handling needs to be adapted to Avalonia's input system
    // private void MainWindow_KeyDown(object? sender, Avalonia.Input.KeyEventArgs e)
    // {
        // if (!interceptReservedKeys) return;

        // var keyBindings = Locator.Current.GetService<SettingsViewModel>()?.KeyBindings;
        // if (keyBindings == null) return;

        // var pressed = e.Key;
        // var modifiers = e.KeyModifiers;

        // // ... (rest of the key handling logic, adapted for Avalonia Key and KeyModifiers) ...
    // }

    // Focus management needs to be adapted
    // private void OnTextBoxGotKeyboardFocus(object sender, KeyboardFocusChangedEventArgs e) => interceptReservedKeys = false;
    // private void OnTextBoxLostKeyboardFocus(object sender, KeyboardFocusChangedEventArgs e) => interceptReservedKeys = true;
}