using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Forms;
using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ImageSort.ViewModels;
using ImageSort.WPF.SettingsManagement;
using ReactiveUI;
using Splat;

namespace ImageSort.WPF.Views;

/// <summary>
///     Interaction logic for FoldersView.xaml
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
                var folderBrowser = new FolderBrowserDialog
                {
                    ShowNewFolderButton = true
                };

                if (folderBrowser.ShowDialog() == DialogResult.OK)
                    ic.SetOutput(folderBrowser.SelectedPath);
            }).DisposeWith(disposableRegistration);

            ViewModel.PromptForName.RegisterHandler(ic =>
            {
                var inputBox = new InputBox(Text.NewFolderPromptText, Text.NewFolderPromptTitle);

                if (inputBox.ShowDialog() == true) ic.SetOutput(inputBox.Answer);
                else ic.SetOutput(null);
            }).DisposeWith(disposableRegistration);

            var currentFolder = new ObservableCollection<FolderTreeItemViewModel>();

            ViewModel.WhenAnyValue(x => x.CurrentFolder)
                .Where(c => c != null)
                .Subscribe(f =>
                {
                    currentFolder.Clear();
                    currentFolder.Add(f);
                })
                .DisposeWith(disposableRegistration);

            var compositeCollection = new CompositeCollection
            {
                new CollectionContainer {Collection = currentFolder},
                new CollectionContainer {Collection = ViewModel.PinnedFolders}
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

            this.BindCommand(ViewModel,
                    vm => vm.CreateFolderUnderSelected,
                    view => view.CreateFolder)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.MoveSelectedPinnedFolderUp,
                    view => view.MoveSelectedPinnedFolderUp)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.MoveSelectedPinnedFolderDown,
                    view => view.MoveSelectedPinnedFolderDown)
                .DisposeWith(disposableRegistration);

            ViewModel.WhenAnyValue(x => x.CurrentFolder)
                .Where(c => c != null)
                .Select(_ => Unit.Default)
                .Subscribe(_ => SelectCurrentFolder())
                .DisposeWith(disposableRegistration);

            var settings = Locator.Current.GetService<SettingsViewModel>();
            var pinnedFolderSettings = settings.GetGroup<PinnedFolderSettingsViewModel>();

            // restore pinned folders
            ViewModel.AddPinnedFoldersFromPaths(pinnedFolderSettings.PinnedFolders);

            // save pinned folders
            ViewModel.PinnedFolders.ActOnEveryObject(f =>
                {
                    if (f == null) return;

                    pinnedFolderSettings.PinnedFolders = ViewModel.PinnedFolders.Select(p => p.Path);
                },
                f =>
                {
                    if (f == null) return;

                    pinnedFolderSettings.PinnedFolders = ViewModel.PinnedFolders.Select(p => p.Path);
                });
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