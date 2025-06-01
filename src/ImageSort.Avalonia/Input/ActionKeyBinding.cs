using System.Windows.Input;

namespace ImageSort.Avalonia.Input;

public class ActionKeyBinding
{
    public string ActionName { get; }
    public Hotkey Hotkey { get; set; } // Made settable for reassignment
    public ICommand Command { get; }
    public object? CommandParameter { get; }

    public ActionKeyBinding(string actionName, Hotkey hotkey, ICommand command, object? commandParameter = null)
    {
        ActionName = actionName;
        Hotkey = hotkey;
        Command = command;
        CommandParameter = commandParameter;
    }
}
