using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System.Collections.Generic;

namespace ImageSort.WPF.SettingsManagement
{
    public class PinnedFolderSettingsViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "PinnedFolders";

        public override string Header => Text.PinnedFoldersSettingsHeader;

        public override bool IsVisible => false;

        private IEnumerable<string> _pinnedFolders = new List<string>();

        public IEnumerable<string> PinnedFolders
        {
            get => _pinnedFolders;
            set => this.RaiseAndSetIfChanged(ref _pinnedFolders, value);
        }
    }
}
