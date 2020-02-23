using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Net.Cache;
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
            });
        }

        public BitmapImage PathToImage(string path)
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
