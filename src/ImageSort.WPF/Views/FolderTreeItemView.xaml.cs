using System;
using System.Collections.Generic;
using System.IO;
using System.Reactive.Disposables;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using ImageSort.ViewModels;
using ImageSort.WPF.FolderIcons;
using ReactiveUI;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for FolderTreeItemView.xaml
    /// </summary>
    public partial class FolderTreeItemView : ReactiveUserControl<FolderTreeItemViewModel>
    {
        public FolderTreeItemView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                this.OneWayBind(ViewModel,
                        vm => vm.FolderName,
                        view => view.FolderName)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                        vm => vm.IsCurrentFolder,
                        view => view.FolderName.FontWeight,
                        current => current ? FontWeights.Bold : FontWeights.Normal)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                        vm => vm.Path,
                        view => view.FolderIcon.Source,
                        path => !Directory.Exists(path) ? null : ShellFileLoader.GetThumbnailFromShellForWpf(path))
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                        vm => vm.IsVisible,
                        view => view.IsVisible)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
