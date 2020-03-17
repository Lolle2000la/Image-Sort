﻿using AdonisUI;
using AdonisUI.Controls;
using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ImageSort.ViewModels;
using ImageSort.WPF.SettingsManagement;
using ImageSort.WPF.SettingsManagement.ShortCutManagement;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
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

            var settings = Locator.Current.GetService<SettingsViewModel>();

            Closed += async (o, e) => await settings.SaveAsync().ConfigureAwait(false);

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

                // bind arrow keys
                reservedKeysPressed.Where(k => k == keyBindings.GoLeft)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Images.GoLeft)
                     .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.GoRight)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Images.GoRight)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.Move)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.MoveImageToFolder)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.Delete)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.DeleteImage)
                    .DisposeWith(disposableRegistration);

                // bind Q and E to undo and redo
                reservedKeysPressed.Where(k => k == keyBindings.Undo)
                   .Select(_ => Unit.Default)
                   .InvokeCommand(ViewModel.Actions.Undo)
                   .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.Redo)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Actions.Redo)
                    .DisposeWith(disposableRegistration);

                // bind WASD to traversing the folders
                reservedKeysPressed
                    .Where(k => k == keyBindings.FolderUp || k == keyBindings.FolderLeft || k == keyBindings.FolderDown || k == keyBindings.FolderRight)
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
                reservedKeysPressed.Where(k => k == keyBindings.OpenFolder)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.OpenFolder)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.OpenSelectedFolder)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.OpenCurrentlySelectedFolder)
                  .DisposeWith(disposableRegistration);

                // bind 'p' and 'u' to pin and unpin
                reservedKeysPressed.Where(k => k == keyBindings.Pin)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.Pin)
                  .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.PinSelected)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.PinSelected)
                  .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.Unpin)
                  .Select(_ => Unit.Default)
                  .InvokeCommand(ViewModel.Folders.UnpinSelected)
                  .DisposeWith(disposableRegistration);

                // bind 'i' to focusing the images search box
                reservedKeysPressed.Where(k => k == keyBindings.SearchImages)
                  .Select(_ => Unit.Default)
                  .Subscribe(_ => Images.SearchTerm.Focus())
                  .DisposeWith(disposableRegistration);

                // bind 'c' to folder creation
                reservedKeysPressed.Where(k => k == keyBindings.CreateFolder)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Folders.CreateFolderUnderSelected)
                    .DisposeWith(disposableRegistration);

                reservedKeysPressed.Where(k => k == keyBindings.Rename)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.Images.RenameImage)
                    .DisposeWith(disposableRegistration);
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

        private void OnOpenSettingsClicked(object sender, RoutedEventArgs e)
        {
            new SettingsView().ShowDialog();
        }

        private void OnOpenKeybindingsClicked(object sender, RoutedEventArgs e)
        {
            new AdonisWindow() 
            { 
                Title = Text.KeyBindingsSettingsHeader,
                Content = new ScrollViewer() { Content = new KeyBindingsSettingsGroupView() },
                Width = 640,
                SizeToContent = SizeToContent.Height
            }.Show();
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

        #endregion IViewFor implementation
    }
}