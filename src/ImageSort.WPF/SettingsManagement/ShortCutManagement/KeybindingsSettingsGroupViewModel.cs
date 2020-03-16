using ImageSort.Localization;
using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
    class KeyBindingsSettingsGroupViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "KeyBindings";

        public override string Header => Text.KeyBindingsSettingsHeader;
    }
}
