using System;
using System.Collections.Generic;
using System.IO;
using System.Reactive.Disposables;
using System.Reactive.Linq;
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

namespace ImageSort.WPF.Views;

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
                    view => view.FolderName.Text)
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

            this.WhenAnyValue(x => x.ActualHeight, x => x.ActualWidth)
                .Select(x => x.Item1 > 0 && x.Item2 > 0)
                .Where(_ => ViewModel != null)
                .Subscribe(v => ViewModel.IsVisible = v);
        });
    }
}
