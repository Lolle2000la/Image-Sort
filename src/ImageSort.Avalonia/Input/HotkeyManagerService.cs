using Avalonia.Input;
using System.Collections.Generic;
using System.Linq;

namespace ImageSort.Avalonia.Input;

public class HotkeyManagerService
{
    private readonly List<Hotkey> _hotkeys;

    public HotkeyManagerService()
    {
        _hotkeys = GetDefaultHotkeys();
    }

    public AppAction? GetActionFor(Key key, KeyModifiers modifiers)
    {
        var hotkey = _hotkeys.FirstOrDefault(h => h.Key == key && h.Modifiers == modifiers);
        return hotkey?.Action;
    }

    // TODO: Implement loading and saving of custom hotkeys
    // public void LoadCustomHotkeys(string filePath) { ... }
    // public void SaveCustomHotkeys(string filePath) { ... }

    private List<Hotkey> GetDefaultHotkeys()
    {
        return new List<Hotkey>
        {
            // Image Navigation
            new Hotkey(Key.Right, KeyModifiers.None, AppAction.NextImage),
            new Hotkey(Key.Left, KeyModifiers.None, AppAction.PreviousImage),

            // Action History
            new Hotkey(Key.Q, KeyModifiers.Control, AppAction.Undo),
            new Hotkey(Key.E, KeyModifiers.Control, AppAction.Redo),

            // Image Operations
            new Hotkey(Key.Up, KeyModifiers.None, AppAction.MoveImageToCurrentSelectedFolder),
            new Hotkey(Key.Down, KeyModifiers.None, AppAction.DeleteImage),
            new Hotkey(Key.F2, KeyModifiers.None, AppAction.RenameImage),

            // Folder Tree Navigation/Selection
            new Hotkey(Key.S, KeyModifiers.None, AppAction.SelectNextFolderInTree),
            new Hotkey(Key.W, KeyModifiers.None, AppAction.SelectPreviousFolderInTree),
            new Hotkey(Key.D, KeyModifiers.None, AppAction.ExpandSelectedTreeFolder),
            new Hotkey(Key.A, KeyModifiers.None, AppAction.CollapseSelectedTreeFolderOrGoToParent),
            new Hotkey(Key.Enter, KeyModifiers.None, AppAction.SetSelectedTreeFolderAsCurrent),

            // Pinned Folder Image Move Operations
            new Hotkey(Key.D1, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder1),
            new Hotkey(Key.D2, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder2),
            new Hotkey(Key.D3, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder3),
            new Hotkey(Key.D4, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder4),
            new Hotkey(Key.D5, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder5),
            new Hotkey(Key.D6, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder6),
            new Hotkey(Key.D7, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder7),
            new Hotkey(Key.D8, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder8),
            new Hotkey(Key.D9, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder9),
            new Hotkey(Key.D0, KeyModifiers.Control, AppAction.MoveImageToPinnedFolder0),

            // UI Control/Focus
            new Hotkey(Key.F, KeyModifiers.Control, AppAction.FocusSearchBox),
            new Hotkey(Key.M, KeyModifiers.Control, AppAction.ToggleMetadataPanel),

            // Application Level
            new Hotkey(Key.O, KeyModifiers.Control, AppAction.OpenFolderDialog),
            new Hotkey(Key.P, KeyModifiers.None, AppAction.PinCurrentFolder),
            new Hotkey(Key.F, KeyModifiers.None, AppAction.PinSelectedTreeFolder), // Note: 'F' without modifier for pinning selected tree folder
            new Hotkey(Key.U, KeyModifiers.None, AppAction.UnpinSelectedTreeFolder),
            new Hotkey(Key.C, KeyModifiers.Control, AppAction.CreateFolderInSelectedTreeFolder),
        };
    }
}
