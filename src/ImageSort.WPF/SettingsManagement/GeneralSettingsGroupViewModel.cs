﻿using AdonisUI;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Windows;

namespace ImageSort.WPF.SettingsManagement
{
    public class GeneralSettingsGroupViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "General";

        public override string Header => "General";

        private bool _darkMode = false;

        public bool DarkMode
        {
            get => _darkMode;
            set => this.RaiseAndSetIfChanged(ref _darkMode, value);
        }

        private bool _checkForUpdatesOnStartup = false;

        public bool CheckForUpdatesOnStartup
        {
            get => _checkForUpdatesOnStartup;
            set => this.RaiseAndSetIfChanged(ref _checkForUpdatesOnStartup, value);
        }

        private bool _installPrereleaseBuilds = false;

        public bool InstallPrereleaseBuilds
        {
            get => _installPrereleaseBuilds;
            set => this.RaiseAndSetIfChanged(ref _installPrereleaseBuilds, value);
        }

        public GeneralSettingsGroupViewModel()
        {
            void SetDarkMode(bool darkMode)
            {
                ResourceLocator.SetColorScheme(Application.Current.Resources, darkMode ? ResourceLocator.DarkColorScheme : ResourceLocator.LightColorScheme);
            }

            this.WhenAnyValue(vm => vm.DarkMode)
                .Subscribe(SetDarkMode);
        }
    }
}
