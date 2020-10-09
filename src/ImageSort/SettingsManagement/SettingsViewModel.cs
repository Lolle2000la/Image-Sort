using System;
using System.Collections.Generic;
using System.Linq;
using ReactiveUI;
using Splat;

namespace ImageSort.SettingsManagement
{
    public class SettingsViewModel : ReactiveObject
    {
        public IEnumerable<SettingsGroupViewModelBase> SettingsGroups { get; }

public SettingsViewModel(IEnumerable<SettingsGroupViewModelBase> settingsGroups = null)
        {
            SettingsGroups = settingsGroups ?? Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>();
        }

        public TGroup GetGroup<TGroup>() where TGroup : SettingsGroupViewModelBase
        {
            return SettingsGroups.OfType<TGroup>()
                .FirstOrDefault();
        }

        public Dictionary<string, Dictionary<string, object>> AsDictionary()
        {
            return SettingsGroups.ToDictionary(@group => @group.Name, @group => @group.SettingsStore);
        }

        public void RestoreFromDictionary(Dictionary<string, Dictionary<string, object>> dictionary)
        {
            if (dictionary == null) throw new ArgumentNullException(nameof(dictionary));

            foreach (var (group, store) in dictionary)
            {
                var settingsGroup = SettingsGroups.FirstOrDefault(g => g.Name == group);

                if (settingsGroup == null) continue;

                foreach (var (storeKey, storeValue) in store) settingsGroup.SettingsStore[storeKey] = storeValue;

                settingsGroup.UpdatePropertiesFromStore();
            }
        }
    }
}