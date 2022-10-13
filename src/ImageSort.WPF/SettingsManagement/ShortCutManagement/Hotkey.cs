using System;
using System.Diagnostics.CodeAnalysis;
using System.Text;
using System.Windows.Input;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement;

public record Hotkey(Key Key, ModifierKeys Modifiers)
{
    public override string ToString()
    {
        var str = new StringBuilder();

        if (Modifiers.HasFlag(ModifierKeys.Control))
            str.Append("Ctrl + ");
        if (Modifiers.HasFlag(ModifierKeys.Shift))
            str.Append("Shift + ");
        if (Modifiers.HasFlag(ModifierKeys.Alt))
            str.Append("Alt + ");
        if (Modifiers.HasFlag(ModifierKeys.Windows))
            str.Append("Win + ");

        str.Append(Key);

        return str.ToString();
    }
}