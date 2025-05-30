using Avalonia.ReactiveUI;
using ImageSort.ViewModels;
using ReactiveUI;

namespace ImageSort.Avalonia.Views
{
    public partial class FolderTreeItemView : ReactiveUserControl<FolderTreeItemViewModel>
    {
        public FolderTreeItemView()
        {
            InitializeComponent();
            this.WhenActivated(disposables => { });
        }
    }
}
