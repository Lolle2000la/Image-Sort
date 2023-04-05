using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Runtime.CompilerServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using AdonisUI.Controls;
using ImageSort.FileSystem;
using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ImageSort.ViewModels;
using ImageSort.ViewModels.Metadata;
using ImageSort.WPF.FileSystem;
using ImageSort.WPF.SettingsManagement;
using ReactiveUI;
using Splat;
using MessageBox = AdonisUI.Controls.MessageBox;
using MessageBoxImage = AdonisUI.Controls.MessageBoxImage;

namespace ImageSort.WPF.Views;

/// <summary>
///     Interaction logic for ImagesView.xaml
/// </summary>
public partial class ImagesView : ReactiveUserControl<ImagesViewModel>
{
    public ImagesView()
    {
        InitializeComponent();

        var generalSettings = Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
            .Select(s => s as GeneralSettingsGroupViewModel)
            .First(s => s != null);

        var panelSettings = Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
                .Select(s => s as MetadataPanelSettings)
                .First(s => s != null);

        Metadata.ViewModel = new MetadataViewModel(Locator.Current.GetService<IMetadataExtractor>(), Locator.Current.GetService<IFileSystem>(), Locator.Current.GetService<MetadataSectionViewModelFactory>());

        this.WhenActivated(disposableRegistration =>
        {
            this.OneWayBind(ViewModel,
                    vm => vm.SelectedImage,
                    view => view.SelectedImage.Source,
                    s =>
                    {
                        if (Path.GetExtension(s)?.ToUpperInvariant() != ".GIF")
                            WpfAnimatedGif.ImageBehavior.SetAnimatedSource(SelectedImage, null);
                        return ImageLoading.GetImageFromPath(s);
                    })
                .DisposeWith(disposableRegistration);

            // for metadata panel width settings
            MetadataColumn.Width = new GridLength((double)panelSettings.MetadataPanelWidth);

            this.WhenAnyValue(x => x.Metadata.ActualWidth)
                .Subscribe(x =>
                panelSettings.MetadataPanelWidth = (int)x)
                .DisposeWith(disposableRegistration);

            // for gif support
            ViewModel.WhenAnyValue(x => x.SelectedImage)
                .Where(s => Path.GetExtension(s)?.ToUpperInvariant() == ".GIF")
                .Where(_ => generalSettings.AnimateGifs)
                .Select(ImageLoading.GetImageFromPath)
                .Subscribe(x => WpfAnimatedGif.ImageBehavior.SetAnimatedSource(SelectedImage, x))
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                    vm => vm.Images,
                    view => view.Images.ItemsSource)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                        vm => vm.SelectedImage,
                        view => view.Metadata.ViewModel.ImagePath)
                    .DisposeWith(disposableRegistration);

            this.Bind(ViewModel,
                    vm => vm.SelectedIndex,
                    view => view.Images.SelectedIndex)
                .DisposeWith(disposableRegistration);

            this.Bind(ViewModel,
                    vm => vm.SearchTerm,
                    view => view.SearchTerm.Text)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.GoLeft,
                    view => view.GoLeft)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.GoRight,
                    view => view.GoRight)
                .DisposeWith(disposableRegistration);

            this.BindCommand(ViewModel,
                    vm => vm.RenameImage,
                    view => view.Rename)
                .DisposeWith(disposableRegistration);

            ViewModel.PromptForNewFileName.RegisterHandler(ic =>
            {
                var inputBox = new InputBox(Text.RenameImagePromptText, Text.RenameImagePromptTitle);

                if (inputBox.ShowDialog() == true) ic.SetOutput(inputBox.Answer);
                else ic.SetOutput(null);
            });

            ViewModel.NotifyUserOfError.RegisterHandler(ic =>
            {
                var messageBox = new MessageBoxModel
                {
                    Caption = Text.Error,
                    Text = ic.Input,
                    Buttons = new[] {MessageBoxButtons.Ok(Text.OK)},
                    Icon = MessageBoxImage.Error
                };

                MessageBox.Show(messageBox);

                ic.SetOutput(Unit.Default);
            });

            ViewModel.GoLeft
                .Merge(ViewModel.GoRight)
                .Subscribe(_ =>
                {
                    if (Images.ItemContainerGenerator.ContainerFromItem(Images.SelectedItem) is ListBoxItem item)
                        item.Focus();
                })
                .DisposeWith(disposableRegistration);
        });
    }

    private void OnSelectedImageChanged(object sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.Count > 0)
            Images.ScrollIntoView(e.AddedItems[0]);
    }
}