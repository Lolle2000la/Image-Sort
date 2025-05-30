using ImageSort.SettingsManagement;
using ImageSort.Localization;
using ReactiveUI;

namespace ImageSort.Avalonia.SettingsManagement
{
    public class MetadataPanelSettings : SettingsGroupViewModelBase
    {
        public override string Name => "MetadataPanel";
        public override string Header => Text.MetadataPanelHeader;
        public override bool IsVisible => false;

        private bool _isExpanded = false;
        public bool IsExpanded
        {
            get => _isExpanded;
            set => this.RaiseAndSetIfChanged(ref _isExpanded, value);
        }

        private int _metadataPanelWidth = 300;
        public int MetadataPanelWidth
        {
            get => _metadataPanelWidth;
            set => this.RaiseAndSetIfChanged(ref _metadataPanelWidth, value);
        }
    }
}
