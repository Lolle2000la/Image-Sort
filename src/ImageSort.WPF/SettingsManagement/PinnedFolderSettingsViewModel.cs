using ImageSort.Localization;
using ImageSort.SettingsManagement;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.WPF.SettingsManagement
{
    public class PinnedFolderSettingsViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "PinnedFolders";

        public override string Header => Text.PinnedFoldersSettingsHeader;
    }
}
