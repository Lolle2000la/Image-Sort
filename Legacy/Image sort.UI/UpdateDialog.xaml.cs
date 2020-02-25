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
using Image_sort.Communication;
using Image_sort.UI.LocalResources.AppResources;
using MahApps.Metro.Controls.Dialogs;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for UpdateDialog.xaml
    /// </summary>
    public partial class UpdateDialog : CustomDialog
    {
        public UpdateDialog()
        {
            InitializeComponent();
        }

        /// <summary>
        /// Sets the changelog formatted in markdown to be shown to the user.
        /// </summary>
        public string ChangelogMarkdown
        {
            set
            {
                ChangelogViewer.Markdown = value;
            }
        }

        /// <summary>
        /// Sets the version of the update to be shown to the user.
        /// </summary>
        public string Version
        {
            set
            {
                VersionText.Text = $"{AppResources.UpdateVersion}: {value}";
                VersionText.Visibility = Visibility.Visible;
            }
        }
    }
}
