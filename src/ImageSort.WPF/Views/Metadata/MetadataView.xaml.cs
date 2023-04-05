using ImageSort.ViewModels.Metadata;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace ImageSort.WPF.Views.Metadata;

/// <summary>
/// Interaction logic for MetadataView.xaml
/// </summary>
public partial class MetadataView : ReactiveUserControl<MetadataViewModel>
{
    public MetadataView()
    {
        InitializeComponent();

        this.WhenActivated(disposableRegistration =>
        {
            this.OneWayBind(ViewModel,
                    vm => vm.Metadata.Type,
                    view => view.IsEnabled,
                    type => type is MetadataResultType.Success)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                    vm => vm.Metadata.Type,
                    view => view.Visibility,
                    type => type is MetadataResultType.Success ? Visibility.Visible : Visibility.Collapsed)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                    vm => vm.SectionViewModels,
                    view => view.Directories.ItemsSource)
                .DisposeWith(disposableRegistration);
        });
    }
}
