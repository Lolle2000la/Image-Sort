using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Reactive.Disposables;
using System.Windows.Forms;

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : ReactiveWindow<MainViewModel>
    {
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
                });
            });
        }
    }
}
