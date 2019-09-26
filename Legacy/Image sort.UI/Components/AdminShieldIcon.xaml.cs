using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using Image_sort.UI.Classes;

namespace Image_sort.UI.Components
{
    /// <summary>
    /// Interaction logic for AdminShieldIcon.xaml
    /// </summary>
    public partial class AdminShieldIcon : UserControl
    {
        public AdminShieldIcon()
        {
            InitializeComponent();
        }

        private void OnAdminShieldIconLoaded(object sender, RoutedEventArgs e)
        {
            shieldIcon.Source = NativeHelpers.AdminSymbol;
        }
    }
}
