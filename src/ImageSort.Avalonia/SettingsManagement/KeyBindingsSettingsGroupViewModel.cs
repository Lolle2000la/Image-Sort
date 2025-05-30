using ImageSort.SettingsManagement;
using ImageSort.Localization;
using ReactiveUI;
using Avalonia.Input;

namespace ImageSort.Avalonia.SettingsManagement
{
    public record Hotkey(Key Key, KeyModifiers Modifiers);

    public class KeyBindingsSettingsGroupViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "KeyBindings";
        public override string Header => Text.KeyBindingsSettingsHeader;

        private Hotkey _move = new Hotkey(Key.Up, KeyModifiers.None);
        public Hotkey Move
        {
            get => _move;
            set => this.RaiseAndSetIfChanged(ref _move, value);
        }

        private Hotkey _delete = new Hotkey(Key.Down, KeyModifiers.None);
        public Hotkey Delete
        {
            get => _delete;
            set => this.RaiseAndSetIfChanged(ref _delete, value);
        }

        private Hotkey _rename = new Hotkey(Key.R, KeyModifiers.None);
        public Hotkey Rename
        {
            get => _rename;
            set => this.RaiseAndSetIfChanged(ref _rename, value);
        }
        
        // Add other hotkeys as needed, mirroring the WPF version but using Avalonia types.
        // For example:
        private Hotkey _goLeft = new Hotkey(Key.Left, KeyModifiers.None);
        public Hotkey GoLeft
        {
            get => _goLeft;
            set => this.RaiseAndSetIfChanged(ref _goLeft, value);
        }

        private Hotkey _goRight = new Hotkey(Key.Right, KeyModifiers.None);
        public Hotkey GoRight
        {
            get => _goRight;
            set => this.RaiseAndSetIfChanged(ref _goRight, value);
        }
    }
}
