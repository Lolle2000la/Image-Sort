using ImageSort.ViewModels;
using ReactiveUI;
using System;
using System.Collections.Generic;
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
