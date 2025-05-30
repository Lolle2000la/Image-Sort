using Avalonia.ReactiveUI;
using ImageSort.ViewModels;

namespace ImageSort.Avalonia.Views;

public partial class FoldersView : ReactiveUserControl<FoldersViewModel>
{
    public FoldersView()
    {
        InitializeComponent();
    }
}
