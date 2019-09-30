using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
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

            Current.Items.Add("");

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

        private void Current_Expanded(object sender, RoutedEventArgs e)
        {
            if (Current.Items.Count == 1 && Current.Items[0] is string)
            {
                Current.Items.Clear();

                ViewModel.IsExpanded = true;
            }
        }
    }
}
