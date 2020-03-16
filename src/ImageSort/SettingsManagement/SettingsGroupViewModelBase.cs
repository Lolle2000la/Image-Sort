using ReactiveUI;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace ImageSort.SettingsManagement
{
    public abstract class SettingsGroupViewModelBase : ReactiveObject
    {
        public abstract string Header { get; }
        protected Dictionary<string, object> SettingsStore { get; } = new Dictionary<string, object>();
        public IReadOnlyDictionary<string, object> SettingsModel => new ReadOnlyDictionary<string, object>(SettingsStore);
    }
}