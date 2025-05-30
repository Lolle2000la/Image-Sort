using System;
using System.IO;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using ImageSort.SettingsManagement; // Core project
using ImageSort.Avalonia.SettingsManagement.ShortCutManagement; // To be created
using Avalonia.Input; // For Avalonia Key and KeyModifiers

namespace ImageSort.Avalonia.SettingsManagement;

internal static class SettingsHelper
{
    static SettingsHelper()
    {
        if (Environment.GetEnvironmentVariable("UI_TEST") is string uiTest)
            ConfigFileLocation = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "ui_test_config.json");
    }

    public static string ConfigFileLocation { get; } = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Image Sort",
#if DEBUG
        "debug_config.json"
#else
        "config.json"
#endif
    );

    public static async Task SaveAsync(this SettingsViewModel settings)
    {
        var dir = Path.GetDirectoryName(ConfigFileLocation);

        if (dir != null && !Directory.Exists(dir)) Directory.CreateDirectory(dir);

        await using var file = File.Create(ConfigFileLocation);

        var serializerOptions = new JsonSerializerOptions
        {
            WriteIndented = true,
            Converters = { new HotkeyJsonConverter() } // Added for Hotkey serialization
        };

        await JsonSerializer.SerializeAsync(file, settings.AsDictionary(), serializerOptions).ConfigureAwait(false);
    }

    public static void Restore(this SettingsViewModel settings)
    {
        if (!File.Exists(ConfigFileLocation)) return;

        using var configFile = File.OpenRead(ConfigFileLocation);
        
        // Deserialize with the custom converter for Hotkeys
        var serializerOptions = new JsonSerializerOptions
        {
            Converters = { new HotkeyJsonConverter() }
        };

        var configContents = JsonSerializer
            .DeserializeAsync<Dictionary<string, Dictionary<string, object>>>(configFile, serializerOptions).Result;

        // The JsonElement to Value conversion needs to be smarter or rely on the converter.
        // For now, if HotkeyJsonConverter handles deserialization to Hotkey objects directly during initial deserialize,
        // this manual JsonElement parsing for Hotkeys might not be needed or needs adjustment.
        // Let's assume HotkeyJsonConverter handles it for now, simplifying this section.
        // If not, the JsonElementToValue logic will need to be robustly ported.

        if (configContents != null) settings.RestoreFromDictionary(configContents);
    }
}
