using ImageSort.ViewModels.Metadata;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Disposables;
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
/// Interaction logic for MetadataSection.xaml
/// </summary>
public partial class MetadataSectionView : ReactiveUserControl<MetadataSectionViewModel>
{
    public MetadataSectionView()
    {
        InitializeComponent();

        this.WhenActivated(disposableRegistration =>
        {
            this.OneWayBind(ViewModel,
                    vm => vm.Title,
                    view => view.Titel.Text)
                .DisposeWith(disposableRegistration);

            this.OneWayBind(ViewModel,
                    vm => vm.FieldViewModels,
                    view => view.Fields.ItemsSource)
                .DisposeWith(disposableRegistration);
        });
    }
}
