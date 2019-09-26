using System;
using System.Collections.Generic;
using System.Diagnostics;
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
    /// Interaction logic for ThumbnailedListBox.xaml
    /// </summary>
    public partial class ThumbnailedListBoxItem : ListBoxItem
    {
        public ThumbnailedListBoxItem()
        {
            InitializeComponent();
            DataContext = this;
        }
        
        /// <summary>
        /// Gets or sets the file property, loads a thumbnail from that from the shell.
        /// note that this is async.
        /// </summary>
        public string File
        {
            get { return (string) GetValue(FileProperty); }
            set { SetValue(FileProperty, value); }
        }

        // Using a DependencyProperty as the backing store for File.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty FileProperty =
            DependencyProperty.Register("File", typeof(string), typeof(ThumbnailedListBoxItem), 
                new PropertyMetadata("", (DependencyObject dp, DependencyPropertyChangedEventArgs e) 
                                            => ((ThumbnailedListBoxItem)dp).OnFileChangedBubbler()));

        /// <summary>
        /// Gets or sets the display name shown to the user, overridden when setting file.
        /// </summary>
        public string DisplayName
        {
            get { return (string) GetValue(DisplayNameProperty); }
            set { SetValue(DisplayNameProperty, value); }
        }

        // Using a DependencyProperty as the backing store for DisplayName.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty DisplayNameProperty =
            DependencyProperty.Register("DisplayName", typeof(string), typeof(ThumbnailedListBoxItem), null);



        /// <summary>
        /// Bubbles the <see cref="FileProperty"/> changing callback to the event handler.
        /// </summary>
        internal void OnFileChangedBubbler()
        {
            OnFilePropertyChanged();
        }

        /// <summary>
        /// Handles the <see cref="File"/> property changing, in order to load the files thumbnail
        /// from the shell.
        /// </summary>
        protected async virtual void OnFilePropertyChanged()
        {
            // set the display name in order to always show the right file and three dots when pointing
            // to a host folder
            DisplayName =System.IO.Path.GetFileName(File);

            try
            {
                // get the fitting thumbnail async from the shell
                BitmapImage thumbnail = (await ShellFileLoader.GetThumbnailFromShellAsync(File)).ToBitmapImage();

                // check if the image is frozen, and load it, if it is.
                if ((bool) thumbnail?.IsFrozen)
                    await Dispatcher.InvokeAsync(() => ThumbnailImage.Source = thumbnail);
            }
            // track failures to load a folder.
            catch (Exception ex)
            {
                Debug.WriteLine(ex.Message);
            }
        }
    }
}
