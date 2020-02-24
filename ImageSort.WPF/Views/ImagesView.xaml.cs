using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
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

                this.BindCommand(ViewModel,
                    vm => vm.GoLeft,
                    view => view.GoLeft)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.GoRight,
                    view => view.GoRight)
                    .DisposeWith(disposableRegistration);

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
    }
}
