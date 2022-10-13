using System;
using System.Windows;
using System.Windows.Controls;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement;

/// <summary>
/// Interaction logic for HotkeyEditor.xaml
/// </summary>
public partial class HotkeyEditor : UserControl
{
    public HotkeyEditor()
    {
        InitializeComponent();
        DataContext = this;
    }

    public string Description
    {
        get { return (string)GetValue(DescriptionProperty); }
        set { SetValue(DescriptionProperty, value); }
    }

    // Using a DependencyProperty as the backing store for Description.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty DescriptionProperty =
        DependencyProperty.Register("Description", typeof(string), typeof(HotkeyEditor), new PropertyMetadata(""));

    public Hotkey Hotkey
    {
        get { return (Hotkey)GetValue(HotkeyProperty); }
        set { SetValue(HotkeyProperty, value); }
    }

    // Using a DependencyProperty as the backing store for Hotkey.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty HotkeyProperty =
        DependencyProperty.Register("Hotkey", typeof(Hotkey), typeof(HotkeyEditor), new PropertyMetadata(null, OnHotkeyChanged));

    private static void OnHotkeyChanged(DependencyObject @object, DependencyPropertyChangedEventArgs args)
    {
        if (@object is HotkeyEditor editor && args.NewValue is Hotkey hotkey)
        {
            editor.HotkeyEditorControl.Hotkey = hotkey;
        }
    }
    private void OnHotkeyEditorControlHotkeyChanged(object sender, EventArgs e)
    {
        Hotkey = (sender as HotkeyEditorControl)?.Hotkey;
    }
}
