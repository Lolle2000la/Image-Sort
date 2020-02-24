using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Linq;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Forms;
using System.Windows.Input;

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : ReactiveWindow<MainViewModel>
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
                        Path = Environment.GetFolderPath(Environment.SpecialFolder.MyPictures)
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
                      var folderBrowser = new FolderBrowserDialog()
                      {
                          ShowNewFolderButton = true
                      };

                      if (folderBrowser.ShowDialog() == System.Windows.Forms.DialogResult.OK)
                          ic.SetOutput(folderBrowser.SelectedPath);
                  })
                  .DisposeWith(disposableRegistration); ;

                  var reservedKeys = new[]
                  {
                    Key.Left, Key.Right, Key.Up, Key.Down,
                    Key.Q, Key.E,
                    Key.W, Key.A, Key.S, Key.D,
                    Key.Enter
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

                  // bind enter
                  reservedKeysPressed.Where(k => k == Key.Enter)
                    .Select(_ => Unit.Default)
                    .InvokeCommand(ViewModel.OpenCurrentlySelectedFolder)
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
    }
}
