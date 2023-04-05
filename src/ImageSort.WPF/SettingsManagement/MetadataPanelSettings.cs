using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.WPF.SettingsManagement;
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

    private int _metadataPanelWidth = 100;
    public int MetadataPanelWidth 
    { 
        get => _metadataPanelWidth; 
        set => this.RaiseAndSetIfChanged(ref _metadataPanelWidth, value); 
    }
}
