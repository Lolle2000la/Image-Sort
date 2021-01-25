using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Globalization;
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

namespace ImageSort.WPF.SettingsManagement.LanguageSelection
{
    /// <summary>
    /// Interaction logic for LanguageSettingsGroupView.xaml
    /// </summary>
    public partial class LanguageSettingsGroupView : ReactiveUserControl<LanguageSettingsGroupViewModel>
    {
        public LanguageSettingsGroupView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                this.OneWayBind(ViewModel,
                    vm => vm.AvailableLanguages,
                    view => view.AvailableLanguages.ItemsSource)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.SelectedLanguage,
                    view => view.AvailableLanguages.SelectedItem)
                    .DisposeWith(disposableRegistration);

                this.BindCommand(ViewModel,
                    vm => vm.ApplyLanguage,
                    view => view.Apply)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
