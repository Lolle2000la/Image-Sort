using AdonisUI;
using ImageSort.Localization;
using ImageSort.SettingsManagement;
using Microsoft.Win32;
using ReactiveUI;
using System;
using System.Windows;

namespace ImageSort.WPF.SettingsManagement;

public class GeneralSettingsGroupViewModel : SettingsGroupViewModelBase
{
    public override string Name => "General";

    public override string Header => Text.GeneralSettingsHeader;

    private bool _darkMode = false;

    public bool DarkMode
    {
        get => _darkMode;
        set => this.RaiseAndSetIfChanged(ref _darkMode, value);
    }

    private bool _checkForUpdatesOnStartup = true;

    public bool CheckForUpdatesOnStartup
    {
        get => _checkForUpdatesOnStartup;
        set => this.RaiseAndSetIfChanged(ref _checkForUpdatesOnStartup, value);
    }

    private bool _installPrereleaseBuilds = false;

    public bool InstallPrereleaseBuilds
    {
        get => _installPrereleaseBuilds;
        set => this.RaiseAndSetIfChanged(ref _installPrereleaseBuilds, value);
    }

    private bool _animateGifs = true;

    public bool AnimateGifs
    {
        get => _animateGifs;
        set => this.RaiseAndSetIfChanged(ref _animateGifs, value);
    }

    private bool _animateGifThumbnails = true;

    public bool AnimateGifThumbnails
    {
        get => _animateGifThumbnails;
        set => this.RaiseAndSetIfChanged(ref _animateGifThumbnails, value);
    }

    private bool _showInExplorerContextMenu = CheckForExplorerContextMenu();

    public bool ShowInExplorerContextMenu
    {
        get => _showInExplorerContextMenu;
        set => this.RaiseAndSetIfChanged(ref _showInExplorerContextMenu, value);
    }

    public GeneralSettingsGroupViewModel()
    {
        void SetDarkMode(bool darkMode)
        {
            ResourceLocator.SetColorScheme(Application.Current.Resources, darkMode ? ResourceLocator.DarkColorScheme : ResourceLocator.LightColorScheme);
        }

        this.WhenAnyValue(vm => vm.DarkMode)
            .Subscribe(SetDarkMode);

#if !DEBUG
        this.WhenAnyValue(vm => vm.ShowInExplorerContextMenu)
            .Subscribe(UpdateExplorerContextMenu);
#endif
    }

    private void UpdateExplorerContextMenu(bool show)
    {
        string[] keys = new[]
        {
            @"Software\Classes\Directory\shell\ImageSort",
            @"Software\Classes\Drive\shell\ImageSort",
            @"Software\Classes\Folder\shell\ImageSort"
        };

        foreach (var key in keys)
        {
            if (show)
            {
                using (var registryKey = Registry.CurrentUser.CreateSubKey(key))
                {
                    registryKey.SetValue("", "Open with Image Sort");
                    registryKey.CreateSubKey("command").SetValue("", $"\"{AppDomain.CurrentDomain.BaseDirectory}Image Sort.exe\" \"%L\"");
                    registryKey.SetValue("Icon", $"\"{AppDomain.CurrentDomain.BaseDirectory}Image Sort.exe\"");
                }
            }
            else
            {
                Registry.CurrentUser.DeleteSubKeyTree(key, false);
            }
        }
    }

    // This is used to grandfather in users who already have the context menu enabled from using it with the installer.
    private static bool CheckForExplorerContextMenu()
    {
        string[] keys = new[]
        {
            @"Software\Classes\Directory\shell\ImageSort",
            @"Software\Classes\Drive\shell\ImageSort",
            @"Software\Classes\Folder\shell\ImageSort"
        };

        foreach (var key in keys)
        {
            if (Registry.CurrentUser.OpenSubKey(key) != null)
            {
                return true;
            }
        }

        return false;
    }
}
