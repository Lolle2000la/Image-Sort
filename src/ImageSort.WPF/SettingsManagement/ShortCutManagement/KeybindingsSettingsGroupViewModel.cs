using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;
using System.Windows.Input;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
    class KeyBindingsSettingsGroupViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "KeyBindings";

        public override string Header => Text.KeyBindingsSettingsHeader;

        private Hotkey _move = new Hotkey(Key.Up, default(ModifierKeys));

        public Hotkey Move
        {
            get => _move;
            set => this.RaiseAndSetIfChanged(ref _move, value);
        }
    }
}
