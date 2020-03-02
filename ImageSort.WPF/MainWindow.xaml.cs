using AdonisUI;
using AdonisUI.Controls;
using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Linq;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : AdonisWindow, IViewFor<MainViewModel>
    {
        private bool interceptReservedKeys = true;

        public MainWindow()
        {
            InitializeComponent();
            ViewModel = new MainViewModel()
            {
                Folders = new FoldersViewModel()
                {
                    CurrentFolder = new FolderTreeItemViewModel()
                    {
                        // will be replaced with the default path or something
                        Path = Environment.GetCommandLineArgs().ElementAtOrDefault(1) ?? Environment.GetFolderPath(Environment.SpecialFolder.MyPictures)
                    }
                },
                Images = new ImagesViewModel(),
                Actions = new ActionsViewModel()
            };

            this.WhenActivated(disposableRegistration =>
            {
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
                    var folderBrowser = new System.Windows.Forms.FolderBrowserDialog()
                    {
                        ShowNewFolderButton = true
                    };

                    if (folderBrowser.ShowDialog() == System.Windows.Forms.DialogResult.OK)
                        ic.SetOutput(folderBrowser.SelectedPath);
                })
                .DisposeWith(disposableRegistration); ;

                var reservedKeys = new[]
                {
                    Key.Left, Key.Right, Key.Up, Key.Down, // image traversal, moving and deletion
                    Key.W, Key.A, Key.S, Key.D, // tree traversal
                    Key.Q, Key.E, // undo and redo
                    Key.O, Key.Enter, // open a new folder (second one opens the selected one)
                    Key.P, Key.F, Key.U, // Pin and unpin folders
                    Key.I, // Focus images search box
                    Key.C // Create a new folder
                };

                var reservedKeysPressed = this.Events().PreviewKeyDown
                    .Where(_ => interceptReservedKeys)
                    .Where(_ => !(Keyboard.FocusedElement is TextBox))
                    .Where(k => reservedKeys.Contains(k.Key))
                    .Do(k => k.Handled = true)
                    .Select(k => k.Key);

                // bind arrow keys
                reservedKeysPressed.Where(k => k == Key.Left)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Images.GoLeft)
                     .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.Right)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Images.GoRight)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.Up)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.MoveImageToFolder)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.Down)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.DeleteImage)
                    .DisposeWith(disposableRegistration);

                // bind Q and E to undo and redo
                reservedKeysPressed.Where(k => k == Key.Q)
                   .Select(_ => Unit.Default)
                   .InvokeCommand(ViewModel.Actions.Undo)
                   .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.E)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Actions.Redo)
                    .DisposeWith(disposableRegistration);

                // bind WASD to traversing the folders
                reservedKeysPressed
                    .Where(k => k == Key.W || k == Key.A || k == Key.S || k == Key.D)
                    .Select(k => k switch
                    {
                        Key.W => Key.Up,
                        Key.A => Key.Left,
                        Key.S => Key.Down,
                        Key.D => Key.Right,
                        Key other => other
                    })
                    .Subscribe(FireKeyEventOnFoldersTree)
                    .DisposeWith(disposableRegistration);

                // bind enter and 'r' to opening a new folder
                reservedKeysPressed.Where(k => k == Key.O)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.OpenFolder)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.Enter)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.OpenCurrentlySelectedFolder)
                  .DisposeWith(disposableRegistration);

                // bind 'p' and 'u' to pin and unpin
                reservedKeysPressed.Where(k => k == Key.P)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.Pin)
                  .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.F)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.PinSelected)
                  .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == Key.U)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.UnpinSelected)
                  .DisposeWith(disposableRegistration);

                // bind 'i' to focusing the images search box
                reservedKeysPressed.Where(k => k == Key.I)
                  .Select(_ => Unit.Default)
                  .Subscribe(_ => Images.SearchTerm.Focus())
                  .DisposeWith(disposableRegistration);

                // bind 'c' to folder creation
                reservedKeysPressed.Where(k => k == Key.C)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Folders.CreateFolderUnderSelected)
                    .DisposeWith(disposableRegistration);

                CheckForUpdates.IsChecked = Settings.Default.ShouldCheckForUpdates;
                InstallPrereleaseBuilds.IsChecked = Settings.Default.UpdateToPrereleaseBuilds;
                if (Settings.Default.DarkMode) SetDarkMode(true);
            });
        }

        private void FireKeyEventOnFoldersTree(Key key)
        {
            interceptReservedKeys = false;

            var target = Folders.Folders/*.ItemContainerGenerator.ContainerFromItem(Folders.Folders.Items[0]) as System.Windows.Controls.TreeViewItem*/;
            var routedEvent = Keyboard.PreviewKeyDownEvent; // Event to send

            target.Focus();

            InputManager.Current.ProcessInput(new System.Windows.Input.KeyEventArgs(
                Keyboard.PrimaryDevice,
                PresentationSource.FromVisual(target),
                0,
                key)
            { RoutedEvent = routedEvent });

            interceptReservedKeys = true;
        }

        private void OnToggleDarkMode(object sender, RoutedEventArgs e)
        {
            var darkMode = sender as ToggleButton;

            SetDarkMode(darkMode?.IsChecked == true);

            Settings.Default.DarkMode = darkMode.IsChecked == true;

            Settings.Default.Save();
        }

        private void OnCheckForUpdatesOnStartupClick(object sender, RoutedEventArgs e)
        {
            InstallPrereleaseBuilds.IsEnabled = CheckForUpdates.IsChecked == true;

            Settings.Default.ShouldCheckForUpdates = CheckForUpdates.IsChecked == true;

            Settings.Default.Save();
        }

        private void OnInstallPrereleaseBuildsClick(object sender, RoutedEventArgs e)
        {
            Settings.Default.UpdateToPrereleaseBuilds = InstallPrereleaseBuilds.IsChecked == true;

            Settings.Default.Save();
        }

        private void SetDarkMode(bool darkMode)
        {
            ResourceLocator.SetColorScheme(Application.Current.Resources, darkMode ? ResourceLocator.DarkColorScheme : ResourceLocator.LightColorScheme);
        }

        #region IViewFor implementation
        public static readonly DependencyProperty ViewModelProperty = DependencyProperty
            .Register(nameof(ViewModel), typeof(MainViewModel), typeof(MainWindow), new PropertyMetadata(null));

        public MainViewModel ViewModel
        {
            get => (MainViewModel)GetValue(ViewModelProperty);
            set => SetValue(ViewModelProperty, value);
        }

        object IViewFor.ViewModel
        {
            get => ViewModel;
            set => ViewModel = (MainViewModel)value;
        }
        #endregion
    }
}
