using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Disposables;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

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
                Images = new ImagesViewModel()
            };

            this.WhenActivated(disposableRegistration =>
            {
                this.Bind(ViewModel,
                    vm => vm.Folders,
                    view => view.Folders.ViewModel)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.OpenCurrentlySelectedFolder,
                    view => view.OpenSelectedFolder)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.Images,
                    view => view.Images.ViewModel)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
