using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System.Collections.Generic;
using System.Linq;
using System.Reactive;
using System.Windows.Input;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement;

public class KeyBindingsSettingsGroupViewModel : SettingsGroupViewModelBase
{
    public override string Name => "KeyBindings";

    public override string Header => Text.KeyBindingsSettingsHeader;

    // image manipulation
    private Hotkey _move = new Hotkey(Key.Up, ModifierKeys.None);

    public Hotkey Move
    {
        get => _move;
        set => this.RaiseAndSetIfChanged(ref _move, value);
    }

    private Hotkey _delete = new Hotkey(Key.Down, ModifierKeys.None);

    public Hotkey Delete
    {
        get => _delete;
        set => this.RaiseAndSetIfChanged(ref _delete, value);
    }

    private Hotkey _rename = new Hotkey(Key.R, ModifierKeys.None);

    public Hotkey Rename
    {
        get => _rename;
        set => this.RaiseAndSetIfChanged(ref _rename, value);
    }

    // image selection
    private Hotkey _goLeft = new Hotkey(Key.Left, ModifierKeys.None);

    public Hotkey GoLeft
    {
        get => _goLeft;
        set => this.RaiseAndSetIfChanged(ref _goLeft, value);
    }

    private Hotkey _goRight = new Hotkey(Key.Right, ModifierKeys.None);

    public Hotkey GoRight
    {
        get => _goRight;
        set => this.RaiseAndSetIfChanged(ref _goRight, value);
    }

    // folder manipulation
    private Hotkey _createFolder = new Hotkey(Key.C, ModifierKeys.None);

    public Hotkey CreateFolder
    {
        get => _createFolder;
        set => this.RaiseAndSetIfChanged(ref _createFolder, value);
    }

    // folder traversal
    private Hotkey _folderUp = new Hotkey(Key.W, ModifierKeys.None);

    public Hotkey FolderUp
    {
        get => _folderUp;
        set => this.RaiseAndSetIfChanged(ref _folderUp, value);
    }

    private Hotkey _folderLeft = new Hotkey(Key.A, ModifierKeys.None);

    public Hotkey FolderLeft
    {
        get => _folderLeft;
        set => this.RaiseAndSetIfChanged(ref _folderLeft, value);
    }

    private Hotkey _folderDown = new Hotkey(Key.S, ModifierKeys.None);

    public Hotkey FolderDown
    {
        get => _folderDown;
        set => this.RaiseAndSetIfChanged(ref _folderDown, value);
    }

    private Hotkey _folderRight = new Hotkey(Key.D, ModifierKeys.None);

    public Hotkey FolderRight
    {
        get => _folderRight;
        set => this.RaiseAndSetIfChanged(ref _folderRight, value);
    }

    // undo and redo
    private Hotkey _undo = new Hotkey(Key.Q, ModifierKeys.None);

    public Hotkey Undo
    {
        get => _undo;
        set => this.RaiseAndSetIfChanged(ref _undo, value);
    }

    private Hotkey _redo = new Hotkey(Key.E, ModifierKeys.None);

    public Hotkey Redo
    {
        get => _redo;
        set => this.RaiseAndSetIfChanged(ref _redo, value);
    }

    // open folder
    private Hotkey _openFolder = new Hotkey(Key.O, ModifierKeys.None);

    public Hotkey OpenFolder
    {
        get => _openFolder;
        set => this.RaiseAndSetIfChanged(ref _openFolder, value);
    }

    private Hotkey _openSelectedFolder = new Hotkey(Key.Enter, ModifierKeys.None);

    public Hotkey OpenSelectedFolder
    {
        get => _openSelectedFolder;
        set => this.RaiseAndSetIfChanged(ref _openSelectedFolder, value);
    }

    // pinned folders
    private Hotkey _pin = new Hotkey(Key.P, ModifierKeys.None);

    public Hotkey Pin
    {
        get => _pin;
        set => this.RaiseAndSetIfChanged(ref _pin, value);
    }

    private Hotkey _pinSelected = new Hotkey(Key.F, ModifierKeys.None);

    public Hotkey PinSelected
    {
        get => _pinSelected;
        set => this.RaiseAndSetIfChanged(ref _pinSelected, value);
    }

    private Hotkey _unpin = new Hotkey(Key.U, ModifierKeys.None);

    public Hotkey Unpin
    {
        get => _unpin;
        set => this.RaiseAndSetIfChanged(ref _unpin, value);
    }

    private Hotkey _moveSelectedPinnedFolderUp = new Hotkey(Key.W, ModifierKeys.Control);

    public Hotkey MoveSelectedPinnedFolderUp
    {
        get => _moveSelectedPinnedFolderUp;
        set => this.RaiseAndSetIfChanged(ref _moveSelectedPinnedFolderUp, value);
    }

    private Hotkey _moveSelectedPinnedFolderDown = new Hotkey(Key.S, ModifierKeys.Control);

    public Hotkey MoveSelectedPinnedFolderDown
    {
        get => _moveSelectedPinnedFolderDown;
        set => this.RaiseAndSetIfChanged(ref _moveSelectedPinnedFolderDown, value);
    }

    // focus image search box
    private Hotkey _searchImages = new Hotkey(Key.I, ModifierKeys.None);

    public Hotkey SearchImages
    {
        get => _searchImages;
        set => this.RaiseAndSetIfChanged(ref _searchImages, value);
    }

    public ReactiveCommand<Unit, Unit> RestoreDefaultBindings { get; }

    public KeyBindingsSettingsGroupViewModel()
    {
        var allHotkeyProps = typeof(KeyBindingsSettingsGroupViewModel).GetProperties().Where(p => p.PropertyType == typeof(Hotkey)).ToArray();

        var defaultHotkeys = new Dictionary<string, Hotkey>();

        foreach ((var propName, var value) in allHotkeyProps.Select(p => (p.Name, p.GetValue(this))))
        {
            SettingsStore[propName] = value;

            defaultHotkeys[propName] = (Hotkey) value;
        }

        RestoreDefaultBindings = ReactiveCommand.Create(() =>
        {
            foreach (var prop in allHotkeyProps)
            {
                foreach (var binding in defaultHotkeys)
                {
                    if (prop.Name == binding.Key)
                    {
                        prop.SetValue(this, binding.Value);
                    }
                }
            }
        });
    }
}
