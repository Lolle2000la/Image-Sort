using AdonisUI.Controls;
using ImageSort.Localization;
using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Drawing;
using System.Globalization;
using System.IO;
using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows.Controls;
using System.Windows.Media;
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
                        Buttons = new[] { MessageBoxButtons.Ok(Text.OK) },
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
                        {
                            item.Focus();
                        }
                    })
                    .DisposeWith(disposableRegistration);
            });
        }

        private static ImageSource PathToImage(string path)
        {
            if (path == null) return null;

            try
            {
                var bitmapImage = new BitmapImage();

                bitmapImage.BeginInit();
                bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
                bitmapImage.UriSource = new Uri(path);
                bitmapImage.EndInit();

                return bitmapImage;
            }
            catch (Exception ex)
            {
                var textDrawing = new GeometryDrawing()
                {
                    Geometry = new GeometryGroup()
                    {
                        Children = new GeometryCollection(new[]
                            {
                                new FormattedText(Text.CouldNotLoadImageErrorText
                                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                                        .Replace("{FileName}", Path.GetFileName(path), StringComparison.OrdinalIgnoreCase),
                                    CultureInfo.CurrentCulture,
                                    System.Windows.FlowDirection.LeftToRight,
                                    new Typeface("Segoe UI"),
                                    16,
                                    System.Windows.Media.Brushes.Black, 1)
                                .BuildGeometry(new System.Windows.Point(8, 8))
                            })
                    },
                    Brush = System.Windows.Media.Brushes.Black,
                    Pen = new System.Windows.Media.Pen(System.Windows.Media.Brushes.White, 1)
                };

                return new DrawingImage(textDrawing);
            }
        }

        private void OnSelectedImageChanged(object sender, System.Windows.Controls.SelectionChangedEventArgs e)
        {
            if (e.AddedItems.Count > 0)
                Images.ScrollIntoView(e.AddedItems[0]);
        }
    }
}