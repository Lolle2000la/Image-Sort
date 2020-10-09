using System.Reactive;
using System.Reactive.Disposables;
using AdonisUI.Controls;
using ImageSort.Localization;
using ImageSort.ViewModels;
using ReactiveUI;

namespace ImageSort.WPF.Views
{
    /// <summary>
    ///     Interaction logic for ActionsView.xaml
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

                ViewModel.NotifyUserOfError.RegisterHandler(ic =>
                {
                    var messageBox = new MessageBoxModel
                    {
                        Caption = Text.Error,
                        Text = ic.Input,
                        Icon = MessageBoxImage.Error,
                        Buttons = new[] {MessageBoxButtons.Ok(Text.OK)}
                    };

                    MessageBox.Show(messageBox);

                    ic.SetOutput(Unit.Default);
                });
            });
        }
    }
}