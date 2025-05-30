using Avalonia.Input;
using System;
using System.Text;

namespace ImageSort.Avalonia.SettingsManagement.ShortCutManagement;

public record Hotkey(Key Key, KeyModifiers Modifiers)
{
    public override string ToString()
    {
        var str = new StringBuilder();

        if (Modifiers.HasFlag(KeyModifiers.Control))
            str.Append("Ctrl + ");
        if (Modifiers.HasFlag(KeyModifiers.Shift))
            str.Append("Shift + ");
        if (Modifiers.HasFlag(KeyModifiers.Alt))
            str.Append("Alt + ");
        if (Modifiers.HasFlag(KeyModifiers.Meta)) // Meta is often Windows/Command key in Avalonia
            str.Append("Meta + ");

        str.Append(Key);

        return str.ToString();
    }
}
