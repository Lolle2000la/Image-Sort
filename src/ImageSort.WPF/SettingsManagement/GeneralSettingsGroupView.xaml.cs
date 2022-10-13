using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using System;
using ReactiveUI;

namespace ImageSort.WPF.SettingsManagement;

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
                    vm => vm.AnimateGifs,
                    view => view.ActivateAnimatedGifs.IsChecked)
                .DisposeWith(disposableRegistration);
            
            this.Bind(ViewModel,
                    vm => vm.AnimateGifThumbnails,
                    view => view.ActivateAnimatedGifsInThumbnails.IsChecked)
                .DisposeWith(disposableRegistration);

            // disable the animated gif thumbnail checkbox if animated gifs are disabled
            this.Bind(ViewModel,
                    vm => vm.AnimateGifs,
                    view => view.ActivateAnimatedGifsInThumbnails.IsEnabled)
                .DisposeWith(disposableRegistration);

            // Show the note about changing gif settings
            ViewModel.WhenAnyValue(x => x.AnimateGifs, x => x.AnimateGifThumbnails)
                .Skip(1) // Skip the startup value
                .Subscribe(b => AnimatedGifsSettingsChangeNotice.Visibility = Visibility.Visible)
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