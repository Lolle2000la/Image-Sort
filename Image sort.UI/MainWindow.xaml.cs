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
        /*********************************************************************/
        /*                                                                   */
        /* ATRIBUTES                                                         */
        /*                                                                   */
        /*********************************************************************/

        /// <summary>
        /// Instance of the <see cref="FolderSelector"/>-Class, that
        /// is managing the folder selecting and getting the <see cref="Image"/>s
        /// in that folder.
        /// </summary>
        private FolderSelector folderSelector = new FolderSelector(Properties.Settings.Default.MaxHorizontalResolution);
        
        /// <summary>
        /// Contains a <see cref="List"/> of <see cref="string"/>'s 
        /// being the paths of the folders inside the currently selected folder.
        /// </summary>
        List<string> folders;
        
        /// <summary>
        /// Gets and sets the maximum horizontal resolution from the settings
        /// </summary>
        public int MaxHorizontalResolution
        {
            get
            {
                return Properties.Settings.Default.MaxHorizontalResolution;
            }
            set
            {
                Properties.Settings.Default.MaxHorizontalResolution = value;
                Properties.Settings.Default.Save();
                folderSelector.SetResolution(Properties.Settings.Default.MaxHorizontalResolution);
            }
        }









        /*********************************************************************/
        /*                                                                   */
        /* CONSTRUCTORS                                                      */
        /*                                                                   */
        /*********************************************************************/

        /// <summary>
        /// Initialization method (default right now)
        /// </summary>
        public MainWindow()
        {
            InitializeComponent();
        }








        /*********************************************************************/
        /*                                                                   */
        /* METHODS                                                           */
        /*                                                                   */
        /*********************************************************************/

        /// <summary>
        /// Loads an image into the window
        /// </summary>
        /// <param name="image">The <see cref="Image"/> that should be displayed</param>
        private void LoadImage(ImageSource image)
        {
            PreviewImage.Source = image;
        }

        /// <summary>
        /// Shifts up the selected folder, to one that is visible, in the <see cref="FoldersStack"/>
        /// </summary>
        private void MoveFolderSelectionUp()
        {
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
        }

        /// <summary>
        /// Shifts down the selected folder, to one that is visible, in the <see cref="FoldersStack"/>
        /// </summary>
        private void MoveFolderSelectionDown()
        {
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
        }

        /// <summary>
        /// Gives the user to select a (new) folder, 
        /// only loads other folder if the user wants to,
        /// and the path given is valid.
        /// </summary>
        private void SelectFolder()
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
                    SelectFolder();
                // otherwise load the image and enable the controls, if there is an image
                else
                {
                    Image buffer = folderSelector.GetNextImage();

                    if (buffer != null)
                    {
                        LoadImage(buffer.Source);
                        EnableControls();
                    }
                    else
                    {
                        LoadImage(null);
                        DisableControls();
                    }
                }

            }

            // Make folders on the left up to date
            AddFoldersToFoldersStack();
        }

        /// <summary>
        /// Enters the folder selected by the user,
        /// if it doesn't work, let the user select a new one
        /// </summary>
        private void EnterFolder()
        {

            // If there are folders in the list (meaning in the folder) then do 
            if (folders != null)
            {
                string folderToEnter = folders[FoldersStack.SelectedIndex];
                if (Directory.Exists(folderToEnter))
                {
                    // if the folder could not be selected, let the user select another.
                    if (folderSelector.Select(folderToEnter) == false)
                        SelectFolder();
                    // otherwise load the image and enable the controls, if there is an image
                    else
                    {
                        Image buffer = folderSelector.GetNextImage();

                        if (buffer != null)
                        {
                            LoadImage(buffer.Source);
                            EnableControls();
                        }
                        else
                        {
                            LoadImage(null);
                            DisableControls();
                        }
                    }
                }

                // Brings the folders on the left up to date
                AddFoldersToFoldersStack();
            }
        }

        /// <summary>
        /// Lets the user create a new folder
        /// </summary>
        private void NewFolder()
        {
            // Only runs when it's usable
            if (NewFolderButton.IsEnabled)
            {
                // gets the name of the folder the user wants
                string folderName = Microsoft.VisualBasic.Interaction.InputBox("What name should the folder have", "Create new Folder", "");
                // Makes sure the user inputted something
                if (folderName != "")
                {
                    // Create the actual directory
                    Directory.CreateDirectory(folderSelector.GetCurrentFolderPath() + @"\"
                        + folderName);

                    // Create and add the item to the FoldersStack
                    ListBoxItem folder = new ListBoxItem()
                    {
                        Content = folderName,
                    };

                    // Make it possible to enter the folder by double clicking it
                    folder.MouseDoubleClick += FolderStackItem_DoubleClick;

                    FoldersStack.Items.Add(folder);

                    // Add the whole path to the collection of folders
                    folders.Add(folderSelector.GetCurrentFolderPath() + @"\" + folderName);
                }
            }
        }

        /// <summary>
        /// Brings the folders in the FoldersStack up to date (if possible)
        /// </summary>
        public void AddFoldersToFoldersStack()
        {
            // Show folders to which it can be moved
            if (Directory.Exists(folderSelector.GetCurrentFolderPath()))
            {
                // Clear all the items out of the list
                FoldersStack.Items.Clear();

                // Get every directory in the folder
                folders = Directory.EnumerateDirectories(folderSelector.GetCurrentFolderPath())
                                   .ToList();

                folders.Insert(0, folderSelector.GetCurrentFolderPath() + @"\..");

                //ListBoxItem folderUpwards = new ListBoxItem()
                //{
                //    Content = ".."
                //};

                //FoldersStack.Items.Add(folderUpwards);

                // get every folder out of the collection of folders
                foreach (string folder in folders)
                {
                    // well if it exists...
                    if (Directory.Exists(folder))
                    {
                        // add it for choice with the content of it's file name
                        ListBoxItem listBoxItem
                            = new ListBoxItem
                            {
                                Content = System.IO.Path.GetFileName(folder)
                            };

                        // Make it possible to enter the folder by double clicking it
                        listBoxItem.MouseDoubleClick += FolderStackItem_DoubleClick;

                        // Adds it to the stack
                        FoldersStack.Items.Add(listBoxItem);
                    }
                }

                FoldersStack.SelectedIndex = 0;
            }
        }

        /// <summary>
        /// Enables all the controls beside the <see cref="SelectFolderButton"/>
        /// </summary>
        public void EnableControls()
        {
            SkipFileButton.IsEnabled = true;
            MoveFolderButton.IsEnabled = true;
            NewFolderButton.IsEnabled = true;
            EnterFolderButton.IsEnabled = true;
        }

        /// <summary>
        /// Disables all the controls beside the <see cref="SelectFolderButton"/>
        /// </summary>
        public void DisableControls()
        {
            SkipFileButton.IsEnabled = false;
            MoveFolderButton.IsEnabled = false;
            NewFolderButton.IsEnabled = false;
            // Don't need to disable EnterFolderButton, because there will always be folders to enter
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
        /// Moves the current image to the folder selected and loads the next one
        /// </summary>
        private void DoMove()
        {
            if (folders.Count > 0)
            {
                if (MoveFolderButton.IsEnabled == true)
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

                    // Move the file
                    folderSelector.MoveFileTo(path,
                        folders.ElementAt(FoldersStack.SelectedIndex) + "\\" +
                        System.IO.Path.GetFileName(path));
                }
            }
            else
            {
                System.Windows.MessageBox.Show("No Folders to move to. Create one first!", "Warning",
                    MessageBoxButton.OK, MessageBoxImage.Warning);
            }
        }

        /// <summary>
        /// Lets the user select a resolution for the loaded images
        /// </summary>
        public void SetResolution()
        {
            string response = Microsoft.VisualBasic.Interaction.InputBox("Please set the horizontal resolution.\n\n\n" +
                "Note: Everything equal or smaller to 0, as well as writing \"default\" reverts the resolution to default (1000),\n" +
                "Note: The higher the resolution, the higher the loading times and RAM usage\n\n" +
                "Will be applied on next loading.",
                "Resolution", Properties.Settings.Default.MaxHorizontalResolution.ToString(), -1, -1);
            // Stores the resolution selected by the user
            /* Gets the resolution by the user via an input box, returns a bool whether 
            he inputted a number or not*/
            bool result = int.TryParse(response, out int resolution);
            // If he did, continue
            if (result)
            {
                // if the resolution is higher 0, then save it in the settings file
                if(resolution > 0)
                {
                    MaxHorizontalResolution = resolution;
                }
                // otherwise, revert to default 
                else
                {
                    MaxHorizontalResolution = 1000;
                }
            }
            // If the response is "" or "default, revert to default
            else if (response == "default")
            {
                MaxHorizontalResolution = 1000;
            }
            // If nothing was given back, then don't change anything
            else if (response == "")
            {
                // Clear so that nothing happens
            }
            // If the user did not input valid numbers, than repeat
            else
                SetResolution();
        }

        /// <summary>
        /// Tells the garbage collector to collect garbage, reduces memory usage when called
        /// </summary>
        private void CollectGarbage()
        {
            GC.Collect();
            GC.WaitForPendingFinalizers();
        }








        /*********************************************************************/
        /*                                                                   */
        /* EVENT HANDLERS                                                    */
        /*                                                                   */
        /*********************************************************************/

        /// <summary>
        /// Makes the search bar work, so that items not containing the string
        /// given by the user get collapsed
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SearchBarBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            // Get all items from the ListBox
            foreach (ListBoxItem foldersStackItem in FoldersStack.Items)
            {
                // If the text of the searchbox it is not "" and alphabetic/numeric
                if (SearchBarBox.Text != "" && Regex.IsMatch(SearchBarBox.Text, @"^[a-zA-Z0-9_]+$"))
                {
                    // and if the item currently looped through doesn't contain the text of the searchbox
                    if (!foldersStackItem.Content.ToString().ToLower().Contains(SearchBarBox.Text))
                        // Make it invisible
                        foldersStackItem.Visibility = Visibility.Collapsed;
                    else
                        // otherwise make it viewable
                        foldersStackItem.Visibility = Visibility.Visible;
                }
                else
                    // otherwise make it visible 
                    foldersStackItem.Visibility = Visibility.Visible;
            }
        }

        /// <summary>
        /// When the user double clicks an element in the <see cref="FoldersStack"/>,
        /// then this method is called. It Enters the folder selected.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public void FolderStackItem_DoubleClick(object sender, EventArgs e)
        {
            EnterFolder();
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
            SelectFolder();
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
        /// Called when the <see cref="MoveFolderButton.Click"/>-Event is being raised
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MoveFolderButton_Click(object sender, RoutedEventArgs e)
        {
            DoMove();
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
                    MoveFolderSelectionUp();
                    break;

                // When down key is pressed, move folder selection down
                case Key.Down:
                    MoveFolderSelectionDown();
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

                // Add Text when space has been pressed
                case Key.Space:
                    SearchBarBox.Text += " ";
                    break;

                // Opens Select Folder dialog
                case Key.F2:
                    SelectFolder();
                    break;

                // Opens new folder Dialog
                case Key.F3:
                    NewFolder();
                    break;

                // Opens dialog for resolution preference
                case Key.F4:
                    SetResolution();
                    break;

                // "Enters" the folder
                case Key.Enter:
                    EnterFolder();
                    break;

                // Insert Characters and numbers only
                default:
                    if(Regex.IsMatch(e.Key.ToString(), @"^[a-zA-Z0-9_]+$") && (e.Key.ToString().Count() < 2))
                        SearchBarBox.Text += e.Key.ToString().ToLower();
                    break;
            }
        }

        /// <summary>
        /// Creates new folder when used
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void NewFolderButton_Click(object sender, RoutedEventArgs e)
        {
            NewFolder();
        }
        /// <summary>
        /// Enters the folder in question when called by user
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void EnterFolderButton_Click(object sender, RoutedEventArgs e)
        {
            EnterFolder();
        }

        /// <summary>
        /// Lets the user select a resolution, by that the images should be loaded.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SetHorizontalResolutionButton_Click(object sender, RoutedEventArgs e)
        {
            SetResolution();
        }
    }
}
