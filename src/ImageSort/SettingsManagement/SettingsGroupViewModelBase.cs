using System;
using ReactiveUI;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Reactive.Linq;

namespace ImageSort.SettingsManagement
{
    public abstract class SettingsGroupViewModelBase : ReactiveObject
    {
        /// <summary>
        /// Used for storage. Should not be changed EVER once set. It must also be unique.
        /// </summary>
        public abstract string Name { get; }
        public abstract string Header { get; }
        public Dictionary<string, object> SettingsStore { get; } = new Dictionary<string, object>();

        protected SettingsGroupViewModelBase()
        {
            Changed.Subscribe(args =>
            {
                SettingsStore[args.PropertyName] = args.Sender.GetType().GetProperty(args.PropertyName).GetValue(args.Sender);
            });
        }

        public void UpdatePropertiesFromStore()
        {
            var properties = this.GetType().GetProperties();

            foreach (var property in properties)
            {
                if (SettingsStore.TryGetValue(property.Name, out var setting))
                {
                    property.SetValue(this, setting);
                }
            }
        }
    }
}