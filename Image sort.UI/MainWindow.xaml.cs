using Image_sort.Logic;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Forms;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        /// <summary>
        /// Instance of the <see cref="FolderSelector"/>-Class, that
        /// is managing the folder selecting and getting the <see cref="Image"/>s
        /// in that folder.
        /// </summary>
        private FolderSelector folderSelector = new FolderSelector();

        IEnumerable<string> folders;

        /// <summary>
        /// Initialization method (default right now)
        /// </summary>
        public MainWindow()
        {
            InitializeComponent();
        }

        /// <summary>
        /// Gets called, when the user clicks the "Select Folder" Button in
        /// the tool bar
        /// </summary>
        /// <param name="sender">
        /// the sender-object of the event
        /// (used when generalizing the event for more controls)
        /// </param>
        /// <param name="e">Contains informations about the event</param>
        private void SelectFolderButton_Click(object sender, RoutedEventArgs e)
        {
            // Creates a dialog for the folder to sort
            FolderBrowserDialog folderBrowser = new FolderBrowserDialog()
            {
                Description = "Which folder needs sorting?",
                ShowNewFolderButton = true
            };

            // Shows it and does things if it works out
            if (folderBrowser.ShowDialog() == System.Windows.Forms.DialogResult.OK)
            {
                // if the folder could not be selected, redo the thing
                if (folderSelector.Select(folderBrowser.SelectedPath) == false)
                    SelectFolderButton_Click(sender, e);
                // otherwise load the image and enable the controls, if there is an image
                else
                {
                    Image buffer = folderSelector.GetNextImage();

                    if(buffer != null)
                    {
                        LoadImage(buffer.Source);
                        EnableControls();
                    }
                        
                }
                    
            }

            // Show folders to which it can be moved
            if (Directory.Exists(folderSelector.GetCurrentFolderPath()))
            {
                // Clear all the items out of the list
                FoldersStack.Items.Clear();

                // Get every directory in the folder
                folders = Directory.EnumerateDirectories(folderSelector.GetCurrentFolderPath());

                // get every folder out of the collection of folders
                foreach (string folder in folders)
                {
                    // well if it exists...
                    if (Directory.Exists(folder))
                    {
                        // add it for choice with the content of it's file name
                        ListBoxItem listViewItem
                            = new ListBoxItem
                            {
                                Content = System.IO.Path.GetFileName(folder)
                            };

                        FoldersStack.Items.Add(listViewItem);
                    }
                }
            }
        }

        /// <summary>
        /// Loads an image into the window
        /// </summary>
        /// <param name="image">The <see cref="Image"/> that should be displayed</param>
        private void LoadImage(ImageSource image)
        {
            PreviewImage.Source = image;                
        }

        /// <summary>
        /// Enables all the controls beside the <see cref="SelectFolderButton"/>
        /// </summary>
        public void EnableControls()
        {
            SkipFileButton.IsEnabled = true;
            MoveFolderButton.IsEnabled = true;
        }

        /// <summary>
        /// Disables all the controls beside the <see cref="SelectFolderButton"/>
        /// </summary>
        public void DisableControls()
        {
            SkipFileButton.IsEnabled = false;
            MoveFolderButton.IsEnabled = false;
        }

        /// <summary>
        /// Handles the Skip File buttons click event.
        /// Loads next image when clicked, without performing an action
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SkipFileButton_Click(object sender, RoutedEventArgs e)
        {
            DoSkip();
        }

        /// <summary>
        /// Skips the current image and loads the next one
        /// </summary>
        public void DoSkip()
        {
            if (SkipFileButton.IsEnabled == true)
            {
                // set the preview image to nothing
                PreviewImage.Source = null;
                // get the next image
                Image buffer = folderSelector.GetNextImage();
                // get the next path of the next image
                string path = folderSelector.GetImagePath();

                // if the buffer is not null, load the image
                if (buffer != null)
                    LoadImage(buffer.Source);
                // else disable the controls
                else
                    DisableControls();
            }
        }

        /// <summary>
        /// Called when the <see cref="MoveFolderButton.Click"/>-Event is being raised
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MoveFolderButton_Click(object sender, RoutedEventArgs e)
        {
            DoMove();
        }

        /// <summary>
        /// Moves the current image to the folder selected and loads the next one
        /// </summary>
        private void DoMove()
        {
            if(MoveFolderButton.IsEnabled == true)
            {

                // set the preview image to nothing
                PreviewImage.Source = null;
                // get the next image
                Image buffer = folderSelector.GetNextImage();
                // get the next path of the next image
                string path = folderSelector.GetImagePath();

                // if the buffer is not null, load the image
                if (buffer != null)
                    LoadImage(buffer.Source);
                // else disable the controls
                else
                {
                    DisableControls();
                }

                //File.Move(path, folders.ElementAt(FoldersStack.SelectedIndex) + "\\" + System.IO.Path.GetFileName(path));

                // Move the file
                folderSelector.MoveFileTo(path,
                    folders.ElementAt(FoldersStack.SelectedIndex) + "\\" +
                    System.IO.Path.GetFileName(path));
            }
        }

        /// <summary>
        /// Handles the Keyboard, so that the user is more productive
        /// (Handles all the shortcuts to more productivity)
        /// </summary>
        /// <param name="sender">
        /// Unneeded necessity for this,
        /// there because it's needed for the Key-down-event
        /// </param>
        /// <param name="e">
        /// Contains the informations about what key was pressed and more,
        /// important for the needs of this app
        /// </param>
        private void Window_KeyDown(object sender, System.Windows.Input.KeyEventArgs e)
        {
            switch (e.Key)
            {
                // When up key is pressed, move folder selection up
                case Key.Up:
                    // If the selected item is bigger than 0
                    if (FoldersStack.SelectedIndex > 0)
                        // Move selection up
                        FoldersStack.SelectedIndex -= 1;
                    // If not, go to the end of the file
                    else
                        FoldersStack.SelectedIndex = FoldersStack.Items.Count - 1;

                    // Go through the elements, so that collapsed elements can be skipped
                    while (((ListBoxItem)FoldersStack.SelectedItem).Visibility == Visibility.Collapsed)
                    {
                        // If the selected item is no 0 go up
                        if (FoldersStack.SelectedIndex > 0)
                            FoldersStack.SelectedIndex -= 1;
                        // otherwise go to top
                        else
                            FoldersStack.SelectedIndex = FoldersStack.Items.Count - 1;
                    }
                    break;

                // When down key is pressed, move folder selection down
                case Key.Down:
                    // If the user hasn't reached the end of the list, go down
                    if (FoldersStack.SelectedIndex < FoldersStack.Items.Count - 1)
                        FoldersStack.SelectedIndex += 1;
                    // otherwise go to the beginning
                    else
                        FoldersStack.SelectedIndex = 0;
                    // Basically goes through the elements and makes sure the collapsed ones get skipped when navigating
                    while (((ListBoxItem)FoldersStack.SelectedItem).Visibility == Visibility.Collapsed)
                    {
                        if (FoldersStack.SelectedIndex < FoldersStack.Items.Count - 1)
                            FoldersStack.SelectedIndex += 1;
                        else
                            FoldersStack.SelectedIndex = 0;
                    }
                    break;

                // Move the file when the right key has been pressed to the selected folder.
                case Key.Right:
                    DoMove();
                    break;

                // Skips the file when the left key has been pressed
                case Key.Left:
                    DoSkip();
                    break;

                // When the back button is pressed, remove one char from the search bar.
                // Do that no matter what is focused.
                case Key.Back:
                    if(SearchBarBox.Text.Count() != 0)
                        SearchBarBox.Text = SearchBarBox.Text.Remove(SearchBarBox.Text.Count() - 1);
                    break;

                case Key.Space:
                    SearchBarBox.Text += " ";
                    break;

                case Key.F2:
                    SelectFolderButton_Click(sender, new RoutedEventArgs());
                    break;

                // Insert Characters and numbers only
                default:
                    if(Regex.IsMatch(e.Key.ToString(), @"^[a-zA-Z0-9_]+$") && (e.Key.ToString().Count() < 2))
                        SearchBarBox.Text += e.Key.ToString().ToLower();
                    break;
            }
        }

        /// <summary>
        /// Makes the search bar work, so that items not containing the string
        /// given by the user get collapsed
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SearchBarBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            foreach(ListBoxItem foldersStackItem in FoldersStack.Items)
            {
                if (SearchBarBox.Text != "" && Regex.IsMatch(SearchBarBox.Text, @"^[a-zA-Z0-9_]+$"))
                {
                    if (!foldersStackItem.Content.ToString().ToLower().Contains(SearchBarBox.Text))
                        foldersStackItem.Visibility = Visibility.Collapsed;
                    else
                        foldersStackItem.Visibility = Visibility.Visible;
                }
                else
                    foldersStackItem.Visibility = Visibility.Visible;
            }
        }
    }
}
