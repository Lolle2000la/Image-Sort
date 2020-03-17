using System;
using System.Collections.Generic;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
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
            DependencyProperty.Register("Hotkey", typeof(Hotkey), typeof(HotkeyEditor), new PropertyMetadata(null));
    }
}
