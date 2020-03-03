using ImageSort.ViewModels;
using IPrompt;
using ReactiveUI;
using System;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media.Imaging;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for ImagesView.xaml
    /// </summary>
    public partial class ImagesView : ReactiveUserControl<ImagesViewModel>
    {
        public ImagesView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                this.OneWayBind(ViewModel,
                    vm => vm.SelectedImage,
                    view => view.SelectedImage.Source,
                    PathToImage)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.Images,
                    view => view.Images.ItemsSource)
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

                ViewModel.PromptForNewFileName.RegisterHandler(ic => ic.SetOutput(
                    IInputBox.Show("What name to rename the image to?", "Rename image", System.Windows.MessageBoxImage.Question)));

                ViewModel.NotifyUserOfError.RegisterHandler(ic =>
                {
                    MessageBox.Show(ic.Input, "An error happened while renaming the image", MessageBoxButton.OK, MessageBoxImage.Error);

                    ic.SetOutput(Unit.Default);
                });

                ViewModel.GoLeft
                    .Merge(ViewModel.GoRight)
                    .Subscribe(_ => 
                    { 
                        if (Images.ItemContainerGenerator.ContainerFromItem(Images.SelectedItem) is ListBoxItem item)
                        {
                            item.Focus();
                        } 
                    })
                    .DisposeWith(disposableRegistration);
            });
        }

        private static BitmapImage PathToImage(string path)
        {
            if (path == null) return null; 

            var bitmapImage = new BitmapImage();

            bitmapImage.BeginInit();
            bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
            bitmapImage.UriSource = new Uri(path);
            bitmapImage.EndInit();

            return bitmapImage;
        }

        private void OnSelectedImageChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
        {
            if (e.AddedItems.Count > 0)
                Images.ScrollIntoView(e.AddedItems[0]);
        }
    }
}
