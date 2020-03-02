using ImageSort.ViewModels;
using IPrompt;
using ReactiveUI;
using System;
using System.Collections.ObjectModel;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Forms;

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

                var currentFolder = new ObservableCollection<FolderTreeItemViewModel>();

                ViewModel.WhenAnyValue(x => x.CurrentFolder)
                    .Where(c => c != null)
                    .Subscribe(f => 
                    {
                        currentFolder.Clear();
                        currentFolder.Add(f);
                    })
                    .DisposeWith(disposableRegistration);

                var compositeCollection = new CompositeCollection()
                {
                    new CollectionContainer() { Collection = currentFolder },
                    new CollectionContainer() { Collection = ViewModel.PinnedFolders }
                };

                Folders.ItemsSource = compositeCollection;

                this.Bind(ViewModel,
                    vm => vm.Selected,
                    view => view.Folders.SelectedItem)
                    .DisposeWith(disposableRegistration);

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

                ViewModel.WhenAnyValue(x => x.CurrentFolder)
                    .Where(c => c != null)
                    .Select(_ => Unit.Default)
                    .Subscribe(_ => SelectCurrentFolder())
                    .DisposeWith(disposableRegistration);

                Observable.FromEventPattern<RoutedEventHandler, RoutedEventArgs>(
                    handler => CreateFolder.Click += handler,
                    handler => CreateFolder.Click -= handler)
                    .Select(_ => IInputBox.Show("What name should the folder have?", "Create a folder", MessageBoxImage.Question))
                    .Where(i => !string.IsNullOrEmpty(i))
                    .InvokeCommand(ViewModel.CreateFolderUnderSelected)
                    .DisposeWith(disposableRegistration);
            });
        }

        private void SelectCurrentFolder()
        {
            if (Folders.Items.Count <= 0) return;

            if (Folders.ItemContainerGenerator.ContainerFromItem(Folders.Items[0])
                is TreeViewItem tvi)
            {
                tvi.IsSelected = true;
                tvi.Focus();
            }
        }
    }
}
