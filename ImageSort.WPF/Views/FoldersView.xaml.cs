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
using System.Windows.Forms;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for FoldersView.xaml
    /// </summary>
    public partial class FoldersView : ReactiveUserControl<FoldersViewModel>
    {
        public FoldersView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                ViewModel.SelectFolder.RegisterHandler(ic =>
                {
                    var folderBrowser = new FolderBrowserDialog() 
                    { 
                        ShowNewFolderButton = true
                    };

                    if (folderBrowser.ShowDialog() == DialogResult.OK)
                        ic.SetOutput(folderBrowser.SelectedPath);
                });

                //this.OneWayBind(ViewModel,
                //    vm => vm.AllFoldersTracked,
                //    view => view.Folders.ItemsSource)
                //    .DisposeWith(disposableRegistration);

                //this.Bind(ViewModel,
                //    vm => vm.Selected,
                //    view => view.Folders.SelectedItem)
                //    .DisposeWith(disposableRegistration);

                ViewModel.WhenAnyValue(x => x.AllFoldersTracked)
                    .Subscribe(folders =>
                    {
                        Folders.Items.Clear();

                        foreach (var folder in folders)
                            Folders.Items.Add(new FolderTreeItemView() { ViewModel = folder });
                    });

                this.BindCommand(ViewModel,
                    vm => vm.Pin,
                    view => view.Pin)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.PinSelected,
                    view => view.PinSelected)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.UnpinSelected,
                    view => view.Unpin)
                    .DisposeWith(disposableRegistration);
            });
        }

        /// <summary>
        /// This keeps track of the selected folder, as every other way of binding the SelectedItem property is unwieldy.
        /// </summary>
        private void Folders_SelectedItemChanged(object sender, RoutedPropertyChangedEventArgs<object> e)
        {
            var selected = (e.NewValue as FolderTreeItemView)?.ViewModel;
            if (selected != null) ViewModel.Selected = selected;
        }
    }
}
