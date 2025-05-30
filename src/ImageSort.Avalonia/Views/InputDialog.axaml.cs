using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using System.Threading.Tasks;

namespace ImageSort.Avalonia.Views;

public partial class InputDialog : Window
{
    public string InputText { get; private set; }

    public InputDialog()
    {
        InitializeComponent();
        OkButton.Click += (_, __) => CloseDialog(true);
        CancelButton.Click += (_, __) => CloseDialog(false);
        InputTextBox.KeyDown += (s, e) =>
        {
            if (e.Key == Key.Enter)
            {
                CloseDialog(true);
            }
            else if (e.Key == Key.Escape)
            {
                CloseDialog(false);
            }
        };
    }

    private void CloseDialog(bool success)
    {
        if (success)
        {
            InputText = InputTextBox.Text;
            Close(InputText);
        }
        else
        {
            Close(null);
        }
    }

    // Optional: Method to set initial text or message
    public void SetParameters(string title, string message, string defaultInput = null)
    {
        Title = title;
        MessageTextBlock.Text = message;
        if (defaultInput != null)
        {
            InputTextBox.Text = defaultInput;
        }
        InputTextBox.Focus();
        InputTextBox.SelectAll();
    }

    // Static method to show the dialog easily
    public static async Task<string> ShowAsync(Window parent, string title, string message, string defaultInput = null)
    {
        var dialog = new InputDialog();
        dialog.SetParameters(title, message, defaultInput);
        return await dialog.ShowDialog<string>(parent);
    }
}
