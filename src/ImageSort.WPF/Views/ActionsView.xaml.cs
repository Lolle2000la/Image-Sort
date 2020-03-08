using ImageSort.ViewModels;
using ReactiveUI;
using System.Reactive.Disposables;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for ActionsView.xaml
    /// </summary>
    public partial class ActionsView : ReactiveUserControl<ActionsViewModel>
    {
        public ActionsView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                this.BindCommand(ViewModel,
                    vm => vm.Undo,
                    view => view.Undo)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.Redo,
                    view => view.Redo)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.LastDone,
                    view => view.Undo.ToolTip)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.LastUndone,
                    view => view.Redo.ToolTip)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
