using System.Reactive.Disposables;
using ReactiveUI;

namespace ImageSort.WPF.SettingsManagement
{
    /// <summary>
    ///     Interaction logic for GeneralSettingsGroupView.xaml
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
            });
        }
    }
}