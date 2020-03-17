using ReactiveUI;
using Splat;
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

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
    /// <summary>
    /// Interaction logic for KeyBindingsSettingsGroupView.xaml
    /// </summary>
    public partial class KeyBindingsSettingsGroupView : ReactiveUserControl<KeyBindingsSettingsGroupViewModel>
    {
        public KeyBindingsSettingsGroupView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                ViewModel ??= Locator.Current.GetService<KeyBindingsSettingsGroupViewModel>();

                this.Bind(ViewModel,
                    vm => vm.Move,
                    view => view.Move.Hotkey)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
