using Avalonia.ReactiveUI;
using ImageSort.ViewModels.Metadata;
using ReactiveUI;

namespace ImageSort.Avalonia.Views.Metadata;

public partial class MetadataView : ReactiveUserControl<MetadataViewModel>
{
    public MetadataView()
    {
        InitializeComponent();

        this.WhenActivated(d =>
        {
            // Add bindings here if needed, or rely on XAML bindings
        });
    }
}
