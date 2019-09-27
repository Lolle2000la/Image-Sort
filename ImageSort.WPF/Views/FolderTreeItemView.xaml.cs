using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
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
                    vm => vm.Path,
                    view => view.Current.Header)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.IsExpanded,
                    view => view.Current.IsExpanded)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.Children,
                    view => view.Current.ItemsSource)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
