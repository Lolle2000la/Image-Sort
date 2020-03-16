using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;

namespace ImageSort.WPF.SettingsManagement
{
    internal static class SettingsHelper
    {
        public static string ConfigFileLocation { get; } = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Image Sort", "config.json");

        public static async Task SaveAsync(this SettingsViewModel settings)
        {
            var dir = Path.GetDirectoryName(ConfigFileLocation);

            if (!Directory.Exists(dir)) Directory.CreateDirectory(dir);

            using var file = File.Create(ConfigFileLocation);

            await JsonSerializer.SerializeAsync(file, settings.AsDictionary()).ConfigureAwait(false);
        }

        public static async Task RestoreAsync(this SettingsViewModel settings)
        {
            if (!File.Exists(ConfigFileLocation)) return;

            using var configFile = File.OpenRead(ConfigFileLocation);

            settings.RestoreFromDictionary(await JsonSerializer.DeserializeAsync<Dictionary<string, Dictionary<string, object>>>(configFile));
        }
    }
}
