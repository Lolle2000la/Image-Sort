using ImageSort.Localization;
using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.WPF.SettingsManagement.LanguageSelection
{
    public class LanguageSettingsGroupViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "LanguageSettings";

        public override string Header => Text.Language;
    }
}
