using ReactiveUI;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace ImageSort.SettingsManagement
{
    public abstract class SettingsGroupViewModelBase : ReactiveObject
    {
        /// <summary>
        /// Used for storage. Should not be changed EVER once set. It must also be unique.
        /// </summary>
        public abstract string Name { get; }
        public abstract string Header { get; }
        protected Dictionary<string, object> SettingsStore { get; } = new Dictionary<string, object>();
        public IReadOnlyDictionary<string, object> SettingsModel => new ReadOnlyDictionary<string, object>(SettingsStore);
    }
}