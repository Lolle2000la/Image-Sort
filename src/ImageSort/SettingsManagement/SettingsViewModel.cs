using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive;
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

        public GroupType GetGroup<GroupType>() where GroupType : SettingsGroupViewModelBase
        {
            return SettingsGroups.OfType<GroupType>()
                .FirstOrDefault();
        }

        public Dictionary<Type, Dictionary<string, object>> AsDictionary()
        {
            throw new NotImplementedException();
        }

        public void RestoreFromDictionary(Dictionary<Type, Dictionary<string, object>> dictionary)
        {
            throw new NotImplementedException();
        }
    }
}
