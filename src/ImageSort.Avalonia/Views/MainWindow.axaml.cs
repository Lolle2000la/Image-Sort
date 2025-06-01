using Avalonia;
using Avalonia.Controls;
using Avalonia.Input; // Required for KeyEventArgs
using Avalonia.ReactiveUI; // Required for ReactiveWindow
using ImageSort.Avalonia.Input; // Required for HotkeyManagerService and AppAction
using ImageSort.Avalonia.ViewModels;
using ReactiveUI;
using System;
using System.Reactive.Disposables;
using System.Reactive.Linq; // Required for Observable.FromEventPattern

namespace ImageSort.Avalonia.Views;

public partial class MainWindow : ReactiveWindow<MainWindowViewModel>
{
    private HotkeyManagerService? _hotkeyManagerService; // Changed from HotkeyService

    public MainWindow()
    {
        InitializeComponent();
#if DEBUG
        this.AttachDevTools();
#endif

        this.WhenActivated(disposableRegistration =>
        {
            _hotkeyManagerService = new HotkeyManagerService(); // Instantiate HotkeyManagerService

            if (ViewModel != null)
            {
                // Subscribe to KeyDown event
                Observable.FromEventPattern<KeyEventArgs>(this, nameof(KeyDown))
                    .Subscribe(args => MainWindow_KeyDown(args.Sender, args.EventArgs))
                    .DisposeWith(disposableRegistration);

                // Handle RequestFocusSearchBox interaction
                ViewModel.RequestFocusSearchBox.RegisterHandler(interaction =>
                {
                    // Assuming ImagesView is accessible and contains SearchTermTextBox
                    // This might need adjustment based on your actual view structure.
                    // If ImagesView is a direct child or accessible through a property:
                    var imagesView = this.FindControl<ImagesView>("ImagesView"); // Ensure ImagesView has x:Name="ImagesView"
                    if (imagesView != null)
                    {
                        var searchBox = imagesView.FindControl<TextBox>("SearchTermTextBox");
                        searchBox?.Focus();
                    }
                    interaction.SetOutput(System.Reactive.Unit.Default);
                })
                .DisposeWith(disposableRegistration);
            }
        });
    }

    private void MainWindow_KeyDown(object? sender, global::Avalonia.Input.KeyEventArgs e) // Use global:: to ensure correct KeyEventArgs
    {
        if (ViewModel == null || _hotkeyManagerService == null) return;

        // Check if focus is on a TextBox or similar input control where typing should be preserved.
        // This is a basic check; more sophisticated focus management might be needed.
        if (e.Source is IInputElement inputElement)
        {
            if (inputElement is TextBox || inputElement is AutoCompleteBox) // Add other input types if necessary
            {
                // Allow specific hotkeys even in textboxes (e.g., Ctrl+Z for Undo)
                // For now, let's assume most hotkeys shouldn't trigger if a textbox has focus,
                // unless they are common text editing shortcuts (which we aren't handling here yet)
                // or explicitly designed to work globally.
                // This logic can be refined.
                // For instance, Ctrl+F for FocusSearchBox should work.
                var potentialAction = _hotkeyManagerService.GetActionFor(e.Key, e.KeyModifiers);
                if (potentialAction != AppAction.FocusSearchBox && 
                    potentialAction != AppAction.Undo && 
                    potentialAction != AppAction.Redo) // Allow Undo/Redo
                {
                    // If not a globally desired action like FocusSearchBox, Undo, Redo, don't process other hotkeys.
                    // This prevents, for example, 'W' in a textbox from triggering 'SelectPreviousFolderInTree'.
                    return; 
                }
            }
        }

        var action = _hotkeyManagerService.GetActionFor(e.Key, e.KeyModifiers);

        if (action.HasValue)
        {
            ViewModel.ExecuteAppAction(action.Value);
            e.Handled = true; // Mark as handled if an action was executed
        }
    }
}