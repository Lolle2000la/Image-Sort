using AdonisUI.Controls;
using ImageSort.SettingsManagement;
using ReactiveUI;
using Splat;
using System;
using System.Linq;
using System.Reactive.Linq;
using System.Windows;

namespace ImageSort.WPF.SettingsManagement
{
    /// <summary>
    /// Interaction logic for SettingsView.xaml
    /// </summary>
    public partial class SettingsView : AdonisWindow, IViewFor<SettingsViewModel>
    {
        public SettingsView()
        {
            InitializeComponent();

            this.WhenActivated(async disposableRegistration =>
            {
                ViewModel ??= Locator.Current.GetService<SettingsViewModel>();

                Closed += async (o, e) => await ViewModel.SaveAsync().ConfigureAwait(false);

                ViewModel.WhenAnyValue(vm => vm.SettingsGroups)
                    .Where(gs => gs != null)
                    .Select(gs => gs.Where(g => g.IsVisible))
                    .Subscribe(gs => Groups.ItemsSource = gs);
            });
        }

        #region IViewFor implementation

        public static readonly DependencyProperty ViewModelProperty = DependencyProperty
            .Register(nameof(ViewModel), typeof(SettingsViewModel), typeof(SettingsView), new PropertyMetadata(null));

        public SettingsViewModel ViewModel
        {
            get => (SettingsViewModel)GetValue(ViewModelProperty);
            set => SetValue(ViewModelProperty, value);
        }

        object IViewFor.ViewModel
        {
            get => ViewModel;
            set => ViewModel = (SettingsViewModel)value;
        }

        #endregion IViewFor implementation
    }
}
