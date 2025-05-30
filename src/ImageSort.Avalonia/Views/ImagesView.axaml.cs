using Avalonia.ReactiveUI;
using ImageSort.ViewModels;

namespace ImageSort.Avalonia.Views;

public partial class ImagesView : ReactiveUserControl<ImagesViewModel>
{
    public ImagesView()
    {
        InitializeComponent();
    }
}
