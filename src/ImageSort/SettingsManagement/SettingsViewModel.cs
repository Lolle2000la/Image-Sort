using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.SettingsManagement
{
    public class SettingsViewModel : ReactiveObject
    {
        public IEnumerable<SettingsGroupViewModelBase> SettingsGroups { get; }

        public SettingsViewModel(IEnumerable<SettingsGroupViewModelBase> settingsGroups = null)
        {
            SettingsGroups = settingsGroups ??= Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>();
        }
    }
}
