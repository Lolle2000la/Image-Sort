using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.WPF.SettingsManagement
{
    internal static class SettingsHelper
    {
        public static void Save(this SettingsViewModel settings)
        {
            Settings.Default["Settings"] = settings.AsDictionary();
            Settings.Default.Save();
        }

        public static void Restore(this SettingsViewModel settings)
        {
            settings.RestoreFromDictionary((Dictionary<string, Dictionary<string, object>>)Settings.Default["Settings"]);
        }
    }
}
