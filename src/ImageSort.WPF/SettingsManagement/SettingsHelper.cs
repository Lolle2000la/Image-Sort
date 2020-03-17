﻿using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
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

        public static void Restore(this SettingsViewModel settings)
        {
            if (!File.Exists(ConfigFileLocation)) return;

            using var configFile = File.OpenRead(ConfigFileLocation);

            var configContents = JsonSerializer.DeserializeAsync<Dictionary<string, Dictionary<string, object>>>(configFile).Result;

            foreach (var configGroup in new Dictionary<string, Dictionary<string, object>>(configContents))
            {
                foreach (var config in new Dictionary<string, object>(configGroup.Value))
                {
                    var everyPossibleGetterMethod = typeof(JsonElement).GetMethods().Where(m => m.Name.StartsWith("TryGet", StringComparison.Ordinal));

                    object JsonElementToValue(JsonElement element)
                    {
                        return element switch
                        {
                            JsonElement { ValueKind: JsonValueKind.False } => false,
                            JsonElement { ValueKind: JsonValueKind.True } => true,
                            JsonElement { ValueKind: JsonValueKind.String } e => e.GetString(),
                            JsonElement { ValueKind: JsonValueKind.Number } e => e.GetInt32(),
                            JsonElement { ValueKind: JsonValueKind.Array } e => e.EnumerateArray().Select(JsonElementToValue).ToArray(),
                            _ => null
                        };
                    }

                    configContents[configGroup.Key][config.Key] = JsonElementToValue((JsonElement)config.Value);
                }
            }

            settings.RestoreFromDictionary(configContents);
        }
    }
}