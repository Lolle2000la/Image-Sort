using System;
using System.Diagnostics.CodeAnalysis;
using System.Text;
using System.Windows.Input;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
    public class Hotkey : IEquatable<Hotkey>
    {
        public Hotkey(Key key, ModifierKeys modifiers)
        {
            Key = key;
            Modifiers = modifiers;
        }

        public Key Key { get; }

        public ModifierKeys Modifiers { get; }

        public bool Equals([AllowNull] Hotkey other)
        {
            return this == other;
        }

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

        public static bool operator ==(Hotkey first, Hotkey second)
        {
            return (first?.Key, first?.Modifiers) == (second?.Key, second?.Modifiers);
        }

        public static bool operator !=(Hotkey first, Hotkey second)
        {
            return (first?.Key, first?.Modifiers) != (second?.Key, second?.Modifiers);
        }

        public static bool Equals(Hotkey left, Hotkey right)
        {
            return left == right;
        }

        public override bool Equals(object obj)
        {
            if (obj is Hotkey hotkey) return this == hotkey;

            return base.Equals(obj);
        }
    }
}