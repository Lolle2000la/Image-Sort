using AdonisUI;
using ImageSort.WPF.SettingsManagement;
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
    /// Interaction logic for GeneralSettingsGroupView.xaml
    /// </summary>
    public partial class GeneralSettingsGroupView : ReactiveUserControl<GeneralSettingsGroupViewModel>
    {
        public GeneralSettingsGroupView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                this.Bind(ViewModel,
                    vm => vm.DarkMode,
                    view => view.DarkMode.IsChecked)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.CheckForUpdatesOnStartup,
                    view => view.CheckForUpdates.IsChecked)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.InstallPrereleaseBuilds,
                    view => view.InstallPrereleaseBuilds.IsChecked)
                    .DisposeWith(disposableRegistration);

                this.OneWayBind(ViewModel,
                    vm => vm.CheckForUpdatesOnStartup,
                    view => view.InstallPrereleaseBuilds.IsEnabled)
                    .DisposeWith(disposableRegistration);

                ViewModel.WhenAnyValue(vm => vm.DarkMode)
                    .Subscribe(SetDarkMode);
            });
        }
        private void SetDarkMode(bool darkMode)
        {
            ResourceLocator.SetColorScheme(Application.Current.Resources, darkMode ? ResourceLocator.DarkColorScheme : ResourceLocator.LightColorScheme);
        }
    }
}
