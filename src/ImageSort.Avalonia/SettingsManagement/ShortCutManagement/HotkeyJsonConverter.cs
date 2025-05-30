using Avalonia.Input;
using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace ImageSort.Avalonia.SettingsManagement.ShortCutManagement;

public class HotkeyJsonConverter : JsonConverter<Hotkey>
{
    public override Hotkey Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType != JsonTokenType.StartObject)
        {
            throw new JsonException("Expected StartObject token");
        }

        Key key = default;
        KeyModifiers modifiers = default;

        while (reader.Read())
        {
            if (reader.TokenType == JsonTokenType.EndObject)
            {
                return new Hotkey(key, modifiers);
            }

            if (reader.TokenType != JsonTokenType.PropertyName)
            {
                throw new JsonException("Expected PropertyName token");
            }

            string? propertyName = reader.GetString();
            reader.Read(); // Move to the property value

            switch (propertyName)
            {
                case "Key":
                case "key": // for case-insensitivity from old files
                    if (reader.TryGetInt32(out int keyValue))
                    {
                        key = (Key)keyValue;
                    }
                    else if (Enum.TryParse<Key>(reader.GetString(), true, out Key parsedKey))
                    {
                        key = parsedKey;
                    }
                    else
                    {
                        throw new JsonException($"Could not parse Key value: {reader.GetString()}");
                    }
                    break;
                case "Modifiers":
                case "modifiers": // for case-insensitivity
                    if (reader.TryGetInt32(out int modifiersValue))
                    {
                        modifiers = (KeyModifiers)modifiersValue;
                    }
                    else if (Enum.TryParse<KeyModifiers>(reader.GetString(), true, out KeyModifiers parsedModifiers))
                    {
                        modifiers = parsedModifiers;
                    }
                    else
                    {
                        throw new JsonException($"Could not parse Modifiers value: {reader.GetString()}");
                    }
                    break;
            }
        }
        throw new JsonException("Expected EndObject token");
    }

    public override void Write(Utf8JsonWriter writer, Hotkey value, JsonSerializerOptions options)
    {
        writer.WriteStartObject();
        writer.WriteNumber("Key", (int)value.Key);
        writer.WriteNumber("Modifiers", (int)value.Modifiers);
        writer.WriteEndObject();
    }
}
