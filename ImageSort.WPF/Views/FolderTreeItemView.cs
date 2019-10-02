using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
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
    public partial class FolderTreeItemView : TreeViewItem, IViewFor<FolderTreeItemViewModel>
    {
        public static readonly DependencyProperty ViewModelProperty = DependencyProperty
            .Register(nameof(ViewModel), typeof(FolderTreeItemViewModel), typeof(FolderTreeItemView));

        public FolderTreeItemView() : base()
        {
            Focusable = true;

            this.WhenActivated(disposableRegistration =>
            {
            Expanded += Current_Expanded;

            if (ViewModel.IsExpandable)
                Items.Add("");

            this.OneWayBind(ViewModel,
                vm => vm.Path,
                view => view.Header)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                vm => vm.IsExpanded,
                view => view.IsExpanded)
                .DisposeWith(disposableRegistration);

            //this.OneWayBind(ViewModel,
            //    vm => vm.Children,
            //    view => view.ItemsSource)
            //    .DisposeWith(disposableRegistration);

            ViewModel.WhenAnyValue(x => x.Children)
                .Where(folders => folders != null)
                .Select(subfolders => subfolders.Select(folder => new FolderTreeItemView() { ViewModel = folder }))
                .Subscribe(subfolders => 
                {
                    Items.Clear();

                    foreach (var subfolder in subfolders)
                        Items.Add(subfolder);
                });
            });
        }

        private void Current_Expanded(object sender, RoutedEventArgs e)
        {
            if (Items.Count == 1 && Items[0] is string)
            {
                Items.Clear();

                ViewModel.IsExpanded = true;
            }
        }


        public FolderTreeItemViewModel ViewModel 
        { 
            get => (FolderTreeItemViewModel)GetValue(ViewModelProperty);
            set => SetValue(ViewModelProperty, value);
        }
        object IViewFor.ViewModel 
        { 
            get => ViewModel;
            set => ViewModel = (FolderTreeItemViewModel)value;
        }

        ~FolderTreeItemView()
        {
            Expanded -= Current_Expanded;
        }
    }
}
