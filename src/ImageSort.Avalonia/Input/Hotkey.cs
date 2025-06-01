using Avalonia.Input;
using System.Text;

namespace ImageSort.Avalonia.Input;

public record Hotkey(Key Key, KeyModifiers Modifiers, AppAction Action)
{
    public override string ToString()
    {
        var str = new StringBuilder();
        if (Modifiers.HasFlag(KeyModifiers.Control)) str.Append("Ctrl + ");
        if (Modifiers.HasFlag(KeyModifiers.Shift)) str.Append("Shift + ");
        if (Modifiers.HasFlag(KeyModifiers.Alt)) str.Append("Alt + ");
        if (Modifiers.HasFlag(KeyModifiers.Meta)) str.Append("Meta + "); // For Windows key or Command key
        str.Append(Key.ToString());
        return str.ToString();
    }
}
