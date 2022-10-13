using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Forms;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using AdonisUI.Controls;
using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ImageSort.ViewModels;
using ImageSort.WPF.SettingsManagement;
using ImageSort.WPF.SettingsManagement.ShortCutManagement;
using ImageSort.WPF.Views;
using ImageSort.WPF.Views.Credits;
using ReactiveUI;
using Splat;
using Application = System.Windows.Application;
using KeyEventArgs = System.Windows.Input.KeyEventArgs;
using TextBox = System.Windows.Controls.TextBox;

namespace ImageSort.WPF;

/// <summary>
///     Interaction logic for MainWindow.xaml
/// </summary>
public partial class MainWindow : AdonisWindow, IViewFor<MainViewModel>
{
    private bool interceptReservedKeys = true;

    public MainWindow()
    {
        InitializeComponent();
        ViewModel = new MainViewModel
        {
            Folders = new FoldersViewModel
            {
                CurrentFolder = new FolderTreeItemViewModel
                {
                    // will be replaced with the default path or something
                    Path = Environment.GetCommandLineArgs().ElementAtOrDefault(1) ??
                           Environment.GetFolderPath(Environment.SpecialFolder.MyPictures)
                }
            },
            Images = new ImagesViewModel(),
            Actions = new ActionsViewModel()
        };

        var settings = Locator.Current.GetService<SettingsViewModel>();

        Closed += async (o, e) => await settings.SaveAsync().ConfigureAwait(false);

        this.WhenActivated(disposableRegistration =>
        {
            // restore last window state
            this.RestoreWindowState();

            // ensure window state is saved when closing
            Closing += (o, e) => this.SaveWindowState();

            this.Bind(ViewModel,
                    vm => vm.Folders,
                    view => view.Folders.ViewModel)
                .DisposeWith(disposableRegistration);

            this.Bind(ViewModel,
                    vm => vm.Images,
                    view => view.Images.ViewModel)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                    vm => vm.Actions,
                    view => view.Actions.ViewModel)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.OpenFolder,
                    view => view.OpenFolder)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.OpenCurrentlySelectedFolder,
                    view => view.OpenSelectedFolder)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.MoveImageToFolder,
                    view => view.Move)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.DeleteImage,
                    view => view.Delete)
                .DisposeWith(disposableRegistration);

            ViewModel.PickFolder.RegisterHandler(ic =>
                {
                    var folderBrowser = new FolderBrowserDialog
                    {
                        ShowNewFolderButton = true
                    };

                    if (folderBrowser.ShowDialog() == System.Windows.Forms.DialogResult.OK)
                        ic.SetOutput(folderBrowser.SelectedPath);
                })
                .DisposeWith(disposableRegistration);

            var keyBindings = Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
                .OfType<KeyBindingsSettingsGroupViewModel>()
                .FirstOrDefault();

            var reservedKeys = keyBindings.SettingsStore
                .Select(kv => kv.Value)
                .OfType<Hotkey>();

            var reservedKeysPressed = this.Events().PreviewKeyDown
                .Where(_ => interceptReservedKeys)
                .Where(_ => !(Keyboard.FocusedElement is TextBox))
                .Where(k => reservedKeys.Contains(new Hotkey(k.Key, Keyboard.Modifiers)))
                .Do(k => k.Handled = true)
                .Select(k => new Hotkey(k.Key, Keyboard.Modifiers));

            IObservable<Unit> KeyPressed(Func<Hotkey> key)
            {
                return reservedKeysPressed.Where(k => k == key())
                    .Select(_ => Unit.Default);
            }

            // bind arrow keys
            KeyPressed(() => keyBindings.GoLeft)
                .InvokeCommand(ViewModel.Images.GoLeft)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.GoRight)
                .InvokeCommand(ViewModel.Images.GoRight)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.Move)
                .InvokeCommand(ViewModel.MoveImageToFolder)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.Delete)
                .InvokeCommand(ViewModel.DeleteImage)
                .DisposeWith(disposableRegistration);

            // bind Q and E to undo and redo
            KeyPressed(() => keyBindings.Undo)
                .InvokeCommand(ViewModel.Actions.Undo)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.Redo)
                .InvokeCommand(ViewModel.Actions.Redo)
                .DisposeWith(disposableRegistration);

            // bind WASD to traversing the folders
            reservedKeysPressed
                .Where(k => k == keyBindings.FolderUp || k == keyBindings.FolderLeft ||
                            k == keyBindings.FolderDown || k == keyBindings.FolderRight)
                .Select(k =>
                {
                    if (k == keyBindings.FolderUp) return Key.Up;
                    if (k == keyBindings.FolderLeft) return Key.Left;
                    if (k == keyBindings.FolderDown) return Key.Down;
                    if (k == keyBindings.FolderRight) return Key.Right;
                    return Key.None;
                })
                .Subscribe(FireKeyEventOnFoldersTree)
                .DisposeWith(disposableRegistration);

            // bind enter and 'r' to opening a new folder
            KeyPressed(() => keyBindings.OpenFolder)
                .InvokeCommand(ViewModel.OpenFolder)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.OpenSelectedFolder)
                .InvokeCommand(ViewModel.OpenCurrentlySelectedFolder)
                .DisposeWith(disposableRegistration);

            // bind 'p' and 'u' to pin and unpin
            KeyPressed(() => keyBindings.Pin)
                .InvokeCommand(ViewModel.Folders.Pin)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.PinSelected)
                .InvokeCommand(ViewModel.Folders.PinSelected)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.Unpin)
                .InvokeCommand(ViewModel.Folders.UnpinSelected)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.MoveSelectedPinnedFolderUp)
                .InvokeCommand(ViewModel.Folders.MoveSelectedPinnedFolderUp)
                .DisposeWith(disposableRegistration);

            KeyPressed(() => keyBindings.MoveSelectedPinnedFolderDown)
                .InvokeCommand(ViewModel.Folders.MoveSelectedPinnedFolderDown)
                .DisposeWith(disposableRegistration);

            // bind 'i' to focusing the images search box
            KeyPressed(() => keyBindings.SearchImages)
                .Subscribe(_ => Images.SearchTerm.Focus())
                .DisposeWith(disposableRegistration);

            // bind 'c' to folder creation
            KeyPressed(() => keyBindings.CreateFolder)
                .InvokeCommand(ViewModel.Folders.CreateFolderUnderSelected)
                .DisposeWith(disposableRegistration);

            // bind 'r' to image renaming
            KeyPressed(() => keyBindings.Rename)
                .InvokeCommand(ViewModel.Images.RenameImage)
                .DisposeWith(disposableRegistration);
        });
    }

    private void FireKeyEventOnFoldersTree(Key key)
    {
        interceptReservedKeys = false;

        var target = Folders.Folders;
        var routedEvent = Keyboard.PreviewKeyDownEvent; // Event to send

        target.Focus();

        InputManager.Current.ProcessInput(new KeyEventArgs(
                Keyboard.PrimaryDevice,
                PresentationSource.FromVisual(target),
                0,
                key)
            {RoutedEvent = routedEvent});

        interceptReservedKeys = true;
    }

    private void OnOpenSettingsClicked(object sender, RoutedEventArgs e)
    {
        new SettingsView().ShowDialog();
    }

    private void OnOpenKeybindingsClicked(object sender, RoutedEventArgs e)
    {
        const int distanceFromTop = 50;

        var keyBindings = new AdonisWindow
        {
            Title = Text.KeyBindingsSettingsHeader,
            Content = new ScrollViewer {Content = new KeyBindingsSettingsGroupView()},
            Width = 640,
            SizeToContent = SizeToContent.Height,
            Top = distanceFromTop
        };

        keyBindings.Show();

        // this opens the window on the same screen and makes sure it is not higher than the screen (out of bounds)
        var windowInteropHelper = new WindowInteropHelper(this);
        var screen = Screen.FromHandle(windowInteropHelper.Handle);

        var dpiScale = VisualTreeHelper.GetDpi(keyBindings);

        var realHeight = screen.WorkingArea.Height / dpiScale.DpiScaleX;

        if (keyBindings.Height > realHeight - distanceFromTop)
        {
            keyBindings.SizeToContent = SizeToContent.Manual;

            keyBindings.Height = realHeight - (distanceFromTop * 2);
        }

        keyBindings.Left = screen.WorkingArea.Left + keyBindings.Left;
        keyBindings.Top = screen.WorkingArea.Top + keyBindings.Top;
    }

    protected override void OnClosed(EventArgs e)
    {
        base.OnClosed(e);

        Application.Current.Shutdown();
    }

    private void OnCreditsClicked(object sender, RoutedEventArgs e)
    {
        CreditsWindow.Window.Show();
        CreditsWindow.Window
            .Activate(); // make sure the window ends up in the foreground when already open to avoid confusion
    }

    #region IViewFor implementation

    public static readonly DependencyProperty ViewModelProperty = DependencyProperty
        .Register(nameof(ViewModel), typeof(MainViewModel), typeof(MainWindow), new PropertyMetadata(null));

    public MainViewModel ViewModel
    {
        get => (MainViewModel) GetValue(ViewModelProperty);
        set => SetValue(ViewModelProperty, value);
    }

    object IViewFor.ViewModel
    {
        get => ViewModel;
        set => ViewModel = (MainViewModel) value;
    }

    #endregion IViewFor implementation
}