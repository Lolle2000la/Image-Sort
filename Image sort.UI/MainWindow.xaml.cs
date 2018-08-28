using Image_sort.Logic;
using Image_sort.UI.Dialogs;
using MahApps.Metro.Controls;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Forms;
using System.Windows.Input;
using System.Windows.Media.Imaging;
using Image_sort.UI.LocalResources.AppResources;
using System.Diagnostics;
using MahApps.Metro.Controls.Dialogs;
using Image_sort.Communication;
using System.Text;
using Image_sort.UI.Classes;
using MahApps.Metro;
using System.Windows.Controls.Primitives;
using System.Windows.Media;

namespace Image_sort.UI
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : MetroWindow
    {
        #region Atributes
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
        private IImageManager imageManager = new ImageFolderManager(Properties.Settings.Default.MaxHorizontalResolution);

        /// <summary>
        /// Contains a <see cref="List"/> of <see cref="string"/>'s 
        /// being the paths of the folders inside the currently selected folder.
        /// </summary>
        List<string> folders;

        /// <summary>
        /// Used to prevent image loading when the ProgressSliders value has changed.
        /// </summary>
        private bool loadImageProgressSlider = true;

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
                imageManager.SetResolution(Properties.Settings.Default.MaxHorizontalResolution);
            }
        }

        /// <summary>
        /// Arguments given by the user/caller
        /// </summary>
        Dictionary<string, string> ArgsGiven = new Dictionary<string, string>();

        private bool searchEnabled = false;

        /// <summary>
        /// Controls, whether the search is enabled.
        /// </summary>
        public bool SearchEnabled
        {
            get { return searchEnabled; }
            set
            {
                searchEnabled = value;
                SearchBarBox.IsEnabled = value;
                SearchBarBox.Visibility = value ? Visibility.Visible : Visibility.Collapsed;
                SearchBarBox.Focusable = value;
                if (value)
                {
                    SearchBarBox.Focus();
                }
                else
                {
                    Focus();
                }
                EnableSearchButton.IsChecked = value;
            }
        }

        /// <summary>
        /// Gets the max of the possible images.
        /// </summary>
        public int MaxImages
        {
            get
            {
                (int progress, int max) = imageManager.GetCurrentProgress();
                return max;
            }
        }

        /// <summary>
        /// Gets or sets the index of the image we are at.
        /// </summary>
        public int CurrentIndex
        {
            get
            {
                return imageManager.CurrentIndex;
            }
            set
            {
                imageManager.CurrentIndex = value;
                SetValue(CurrentIndexProperty, value);
            }
        }

        // Using a DependencyProperty as the backing store for CurrentIndex.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty CurrentIndexProperty =
            DependencyProperty.Register("CurrentIndex", typeof(int), typeof(MainWindow), null);

        /// <summary>
        /// Window used for giving the user a little help.
        /// </summary>
        private HelpWindow helpWindow;

        /// <summary>
        /// keeps track of whether the slider progress was changed.
        /// </summary>
        private bool SliderValueChanged = false;

        /// <summary>
        /// This keeps track of how many dialogs are open.
        /// </summary>
        protected int DialogsOpen { get; private set; } = 0;
        #endregion




        #region Constructors
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

            // Upgrade settings if needed.
            if (Properties.Settings.Default.UpgradeRequired)
            {
                Properties.Settings.Default.Upgrade();
                Properties.Settings.Default.UpgradeRequired = false;
                Properties.Settings.Default.Save();
            }

            // Get parameters from the command line
            string[] args = Environment.GetCommandLineArgs();

            // Add them to the dictionary
            for (int index = 1; index < args.Length; index += 2)
            {
                ArgsGiven.Add(args[index], args[index + 1]);
            }

            // loads Folder if one was selected
            // the key for folder is "f"
            if (ArgsGiven.TryGetValue("-f", out string argValue))
            {
                // Load the image at the given path.
                LoadFolderAsync(argValue, true);
            }

#if !DEBUG_WINDOW
            // Sets the size and position of the window to one from the last session.
            Top = Properties.Settings.Default.Top;
            Left = Properties.Settings.Default.Left;
            Height = Properties.Settings.Default.Height;
            Width = Properties.Settings.Default.Width;
            // Very quick and dirty - but it does the job
            if (Properties.Settings.Default.Maximized)
            {
                WindowState = WindowState.Maximized;
            }
#endif

            // Set the resolution to the saved one.
            ResolutionBox.Text = MaxHorizontalResolution.ToString();
            imageManager.SetResolution(MaxHorizontalResolution);

            // Fill in the requiered instance, needed for event bubbling.
            FoldersStack.MainWindowParent = this;

            // Timer used to update the loaded image based on the slider value, if that has changed.
            Timer timer = new Timer {
                Enabled = true,
                // timer should run every 250 seconds.
                Interval = 250
            };
            timer.Tick += async (object s, EventArgs e) =>
            {
                // Only change things if the slider value has changed.
                if (SliderValueChanged)
                {
                    // set SliderValueChanged to false to prevent endless reloads.
                    SliderValueChanged = false;

                    // ensure right execution context.
                    await Dispatcher.Invoke(async () =>
                    {
                        // then load the image and set lastRun to the new Task.
                        await LoadImageFromSliderValue();
                    });
                }
            };

            // called when an folder was changed.
            imageManager.FolderChanged += async (o, e) =>
            {
                // sets the current index back before loading the "next"
                imageManager.CurrentIndex--;

                //// set the preview image to nothing
                //PreviewImage.Source = null;
                // get the next image
                BitmapImage buffer = await imageManager.GetNextImage();
                // get the next path of the next image
                string path = imageManager.GetImagePath();

                // if the buffer is not null, load the image
                if (buffer != null)
                {
                    Dispatcher.Invoke(() =>
                    {
                        LoadImage(buffer);
                        // Enable the controls again.
                        EnableControls();
                    });
                }
                // else disable the controls
                else
                {
                    // and unload it
                    Dispatcher.Invoke(() =>
                    {
                        LoadImage(null);
                        DisableControls();
                        GoBackButton.IsEnabled = true;
                    });
                }

                loadImageProgressSlider = false;
                Dispatcher.Invoke(() => ProgressSlider.Value = imageManager.CurrentIndex - 1);
            };
        }
        #endregion




        #region Methods
        /*********************************************************************/
        /*                                                                   */
        /* METHODS                                                           */
        /*                                                                   */
        /*********************************************************************/

        #region Folder-Selection Management
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
            while (IsAnyFolderVisible && ((ListBoxItem) FoldersStack.SelectedItem).Visibility == Visibility.Collapsed)
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
            while (IsAnyFolderVisible && ((ListBoxItem) FoldersStack.SelectedItem).Visibility == Visibility.Collapsed)
            {
                if (FoldersStack.SelectedIndex < FoldersStack.Items.Count - 1)
                    FoldersStack.SelectedIndex += 1;
                else
                    FoldersStack.SelectedIndex = 0;
            }
        }
        #endregion

        #region Folder-Navigation/Loading
        /// <summary>
        /// Lets the user select a resolution for the loaded images
        /// </summary>
        public void SetResolution()
        {
            string response = InputBox.Show("Please set the horizontal resolution.\n\n\n" +
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
                if (resolution > 0)
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
        /// Selects and loads a folder
        /// </summary>
        /// <param name="folder">The folder that should get selected</param>
        private async Task<bool> SelectAndLoadFolder(string folder)
        {
            DisableAllControls();

            // if the folder could not be selected, redo the thing
            if (!imageManager.SetCurrentFolder(folder))
            {
                // Only enable opening another one
                SelectFolderButton.IsEnabled = true;

                return false;
            }
            // otherwise load the image and enable the controls, if there is an image
            else
            {
                // Show the current folder in the title bar
#if (!DEMO)
                Title = $"Image sort - {imageManager.CurrentFolder}";
#else
                Title = $"Image sort - {Path.GetFileName(imageManager.CurrentFolder)}";
#endif
                // Get the next image
                BitmapImage buffer = await imageManager.GetNextImage();

                // if one was given back, load it (also enable controls)
                if (buffer != null)
                {
                    LoadImage(buffer);
                    EnableControls();
                }
                // otherwise don't (also disable controls)
                else
                {
                    LoadImage(null);
                    DisableControls();
                    ProgressSlider.Maximum = MaxImages;
                    ProgressSlider.Value = 0;
                    return true;
                }
                ProgressSlider.Maximum = MaxImages;
                ProgressSlider.Value = 0;
                EnableAllControls();
                return true;
            }
        }

        /// <summary>
        /// Gives the user to select a (new) folder, 
        /// only loads other folder if the user wants to,
        /// and the path given is valid.
        /// </summary>
        private async void SelectFolder()
        {
            // Creates a dialog for the folder to sort
            FolderBrowserDialog folderBrowser = new FolderBrowserDialog() {
                Description = AppResources.WhichFolderQuestion,
                ShowNewFolderButton = true
            };

            // Shows it and does things if it works out
            if (folderBrowser.ShowDialog() == System.Windows.Forms.DialogResult.OK)
            {
                // Disable all user input to prevent unwanted behavior
                DisableAllControls();

                // if the folder could not be selected, ask the user if he wants to retry.
                // Also clean-up and give the user only the ability to select another folder.
                if (await SelectAndLoadFolder(folderBrowser.SelectedPath) == false)
                {
                    // Clean-Up
                    FoldersStack.Items.Clear();
                    DisableAllControls();
                    SelectFolderButton.IsEnabled = true;
                    ResolutionBox.IsEnabled = true;

                    // Ask the user if he wants to retry
                    if (System.Windows.Forms.MessageBox.Show("Folder could not be opened. " +
                        "The process was either aborted or the folder at the given destination " +
                        "can't be accessed.", "Could not open",
                        MessageBoxButtons.RetryCancel, MessageBoxIcon.Error) == System.Windows.Forms.DialogResult.Retry)
                        SelectFolder();

                    // end the function
                    return;
                }


                // otherwise load the image and enable the controls, if there is an image
                else
                {
                    // if there was no image inside the folder, 
                    // disable anything except for the controls needed for navigation.
                    if (PreviewImage.Source == null)
                    {
                        DisableAllControls();
                        SelectFolderButton.IsEnabled = true;
                        ResolutionBox.IsEnabled = true;
                        EnableSearchButton.IsEnabled = true;
                        EnterFolderButton.IsEnabled = true;

                        // Make folders on the left up to date
                        AddFoldersToFoldersStack();

                        // Clearing the search bar after entering the folder,
                        // so that it will be more comfortable searching.
                        SearchBarBox.Text = "";

                        return;
                    }

                    // Clearing the search bar after entering the folder,
                    // so that it will be more comfortable searching.
                    SearchBarBox.Text = "";

                    // Make folders on the left up to date
                    AddFoldersToFoldersStack();
                }

                // Enable all controls again to allow for user input
                EnableAllControls();
            }
        }

        /// <summary>
        /// Enters the folder selected by the user,
        /// if it doesn't work, let the user select a new one
        /// </summary>
        private async void EnterFolder()
        {

            // If there are folders in the list (meaning in the folder) then do 
            if (folders != null)
            {
                string folderToEnter = Path.GetFullPath(folders[FoldersStack.SelectedIndex]);
                if (Directory.Exists(folderToEnter))
                {
                    // Disable all user input to prevent unwanted behavior
                    DisableAllControls();

                    // if the folder could not be selected, show the user that it couldn't
                    if (await SelectAndLoadFolder(folderToEnter) == false)
                    {
                        // notification
                        System.Windows.Forms.MessageBox.Show("Folder could not be opened. " +
                        "The process was either aborted or the folder at the given destination " +
                        "can't be accessed.", "Could not open",
                        MessageBoxButtons.OK, MessageBoxIcon.Error);

                        // Clean-Up
                        FoldersStack.Items.Clear();
                        DisableAllControls();
                        SelectFolderButton.IsEnabled = true;
                        ResolutionBox.IsEnabled = true;

                    }

                    // otherwise load the image and enable the controls, if there is an image.
                    else
                    {
                        // if there was no image inside the folder, 
                        // disable anything except for the controls needed for navigation.
                        if (PreviewImage.Source == null)
                        {
                            DisableAllControls();
                            SelectFolderButton.IsEnabled = true;
                            ResolutionBox.IsEnabled = true;
                            EnableSearchButton.IsEnabled = true;
                            EnterFolderButton.IsEnabled = true;

                            // Make folders on the left up to date
                            AddFoldersToFoldersStack();

                            // Clearing the search bar after entering the folder,
                            // so that it will be more comfortable searching.
                            SearchBarBox.Text = "";

                            return;
                        }

                        // Clearing the search bar after entering the folder,
                        // so that it will be more comfortable searching.
                        SearchBarBox.Text = "";
                    }

                    // Enable all controls again to allow for user input
                    EnableAllControls();
                }

                // Brings the folders on the left up to date
                AddFoldersToFoldersStack();
            }
        }

        /// <summary>
        /// Loads the folder at the path given, and then adds the folders in that folder
        /// to the selection in the <see cref="FoldersStack"/>
        /// </summary>
        /// <param name="folder">The folder that should be selected/loaded.</param>
        /// <param name="hideMainWindow">
        /// Indicates, whether the main window should be hidden during the loading process.
        /// </param>
        private async void LoadFolderAsync(string folder, bool hideMainWindow = false)
        {
            if (Directory.Exists(folder))
            {
                // Hides the main window before loading.
                if (hideMainWindow)
                    Hide();

                // Load the folder
                if (await SelectAndLoadFolder(folder))
                {
                    // Refresh folders
                    AddFoldersToFoldersStack();
                }
                // if it doesn't work, clean up and only enable opening another one.
                else
                {
                    DisableAllControls();
                    SelectFolderButton.IsEnabled = true;
                    ResolutionBox.IsEnabled = true;
                }

                // Shows the main window after loading is complete.
                if (hideMainWindow)
                    Show();
            }
            else if (folder.EndsWith(".jpg") || folder.EndsWith(".png"))
            {
                // if a image was given, open the host folder.
                string hostFolder = Path.GetDirectoryName(folder);
                if (Directory.Exists(hostFolder))
                    LoadFolderAsync(hostFolder);
            }
        }
        #endregion

        #region Folder Management
        /// <summary>
        /// Lets the user create a new folder
        /// </summary>
        private async void NewFolder()
        {
            // Only runs when it's usable
            if (NewFolderButton.IsEnabled)
            {
                // signal that at least one dialog is open.
                DialogsOpen++;
                // gets the name of the folder the user wants
                string folderName = await this.ShowInputAsync(AppResources.CreateNewFolder,
                                                        AppResources.CreateNewFolderMessage,
                                                        new MetroDialogSettings() {
                                                            AffirmativeButtonText = AppResources.Yes,
                                                            NegativeButtonText = AppResources.No,
                                                            DefaultText = AppResources.NewFolder
                                                        });
                // signal that at least one dialog is not open anymore.
                DialogsOpen--;
                // Makes sure the user inputted something
                if (folderName != null)
                {
                    // Create the actual directory
                    Directory.CreateDirectory(Path.Combine(imageManager.CurrentFolder, folderName));

                    // Create and add the item to the FoldersStack
                    ListBoxItem folder = new ListBoxItem() {
                        Content = folderName,
                    };

                    // Make it possible to enter the folder by double clicking it
                    folder.MouseDoubleClick += FolderStackItem_DoubleClick;

                    FoldersStack.Items.Add(folder);

                    // Add the whole path to the collection of folders
                    folders.Add(Path.Combine(imageManager.CurrentFolder, folderName));
                }
            }
        }

        /// <summary>
        /// Asks the user if he wants to delete the current folder.
        /// </summary>
        private async void DeleteFolder()
        {
            if ((imageManager.CurrentFolder.Length > 3 && FoldersStack.SelectedIndex > 1)
                || (imageManager.CurrentFolder.Length <= 3))
                if (DeleteFolderButton.IsEnabled == true)
                {
                    // get the current image to use it later.
                    string currentFolder = Path.Combine(imageManager.CurrentFolder,
                        ((ListBoxItem) FoldersStack.SelectedItem).Content.ToString());

                    try
                    {
                        DialogsOpen++;
                        if (await this.ShowMessageAsync(AppResources.DeleteFolder,
                                                        AppResources.DeleteFolderConfirmation.Replace("{folder}", currentFolder),
                                                        MessageDialogStyle.AffirmativeAndNegative,
                                                        new MetroDialogSettings() {
                                                            AffirmativeButtonText = AppResources.Delete,
                                                            NegativeButtonText = AppResources.DontDelete
                                                        })
                            == MessageDialogResult.Affirmative)
                            // send image to the recycle bin
                            if (FileOperationApiWrapper.Send(currentFolder,
                                FileOperationApiWrapper.FileOperationFlags.FOF_ALLOWUNDO
                                | FileOperationApiWrapper.FileOperationFlags.FOF_NOCONFIRMATION))
                            {
                                int currentIndex = FoldersStack.SelectedIndex;
                                // remove an folder
                                folders.RemoveAt(currentIndex);
                                FoldersStack.SelectedIndex--;
                                FoldersStack.Items.RemoveAt(currentIndex);
                            }
                        DialogsOpen--;
                    }
                    catch (System.ComponentModel.Win32Exception ex)
                    {
                        await this.ShowMessageAsync(
                                   AppResources.CouldNotDeleteFile.Replace("{filename}", currentFolder),
                                   AppResources.CouldNotDeleteFileMessage.Replace("{filename}", currentFolder)
                                                                         .Replace("{errcode}", ex.NativeErrorCode.ToString())
                                   );
                        DialogsOpen--;
                    }
                }
        }
        #endregion

        #region Data-Refreshing
        /// <summary>
        /// Loads an image into the window
        /// </summary>
        /// <param name="image">The <see cref="Image"/> that should be displayed</param>
        private void LoadImage(BitmapImage image)
        {
            PreviewImage.Source = image;

            // if an image was given, fill in the information (meta-data)
            if (image != null)
            {
                string pathToImage = imageManager.GetImagePath();

                FileNameInfoLabel.Text = $"{AppResources.Name}:";
                FileNameInfo.Text = Path.GetFileNameWithoutExtension(pathToImage);
                FileNameInfo.Visibility = Visibility.Visible;
                FileNameInfo.Focusable = true;
                FileTypeInfo.Text = $"{AppResources.Format}: {Path.GetExtension(pathToImage)}";
                DateTime creationTime = File.GetCreationTime(pathToImage).ToLocalTime();
                FileCreationTimeInfo.Text = $"{AppResources.CreatedAt}: {creationTime.ToLongDateString()} {creationTime.ToShortTimeString()}";
                // Calculates the sizes in MB and KB and rounds them to two digits, before filling in the size.
                FileSizeInfo.Text = $"{AppResources.Size}: " +
                    $"{Math.Round(((double) (new FileInfo(pathToImage)).Length) / (1024 * 1024), 2)} MB, " +
                    $"{Math.Round(((double) (new FileInfo(pathToImage)).Length) / 1024, 2)} KB";

                // Fill in the links destination and make it visible
                OpenInExplorerLink.NavigateUri = new Uri(pathToImage);
                OpenInExplorerLinkHost.Visibility = Visibility.Visible;

                ProgressSlider.Visibility = Visibility.Visible;
                ProgressSlider.Maximum = MaxImages;

                // Show Progress
                (int current, int max) = imageManager.GetCurrentProgress();
                ProgressIndicatorText.Text = $"{AppResources.Progress}: {current}/{max}";
            }
            // if that is not the case, remove the old information.
            else
            {
                FileNameInfoLabel.Text = "";
                FileNameInfo.Text = "";
                FileNameInfo.Visibility = Visibility.Collapsed;
                FileNameInfo.Focusable = false;
                FileTypeInfo.Text = "";
                FileCreationTimeInfo.Text = "";
                FileSizeInfo.Text = "";

                // Show progress
                (int current, int max) = imageManager.GetCurrentProgress();
                ProgressIndicatorText.Text = $"Progress: {current}/{max}";

                OpenInExplorerLink.NavigateUri = null;
                // Collapse link again to prevent accidental clicking.
                OpenInExplorerLinkHost.Visibility = Visibility.Collapsed;
            }
        }

        /// <summary>
        /// Opens explorer.exe (File Explorer) and selects the given file.
        /// </summary>
        /// <param name="path">The file to be selected.</param>
        private void OpenImageInFileExplorer(string path)
        {
            // Opens explorer.exe (File Explorer) and selects the given file.
            System.Diagnostics.Process.Start("explorer.exe", $"/select,\"{path}\"");
        }

        /// <summary>
        /// Brings the folders in the FoldersStack up to date (if possible)
        /// </summary>
        public void AddFoldersToFoldersStack()
        {
            // Show folders to which it can be moved
            if (Directory.Exists(imageManager.CurrentFolder))
            {
                // Clear all the items out of the list
                FoldersStack.Items.Clear();

                // Get every directory in the folder
                folders = Directory.EnumerateDirectories(imageManager.CurrentFolder)
                                   .ToList();

                // only add the .. when there is another parent folder.
                if (Path.GetPathRoot(imageManager.CurrentFolder) != imageManager.CurrentFolder)
                {
                    folders.Insert(0, imageManager.CurrentFolder + @"\..");
                }

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
                            = new ListBoxItem {
                                Content = System.IO.Path.GetFileName(folder),
                            };

                        // Make it possible to enter the folder by double clicking it
                        listBoxItem.MouseDoubleClick += FolderStackItem_DoubleClick;

                        // Adds it to the stack
                        FoldersStack.Items.Add(listBoxItem);
                    }
                }

                FoldersStack.SelectedIndex = 0;
            }
            else
            {
                // Make sure that there are not folders to select if there is none selected.
                FoldersStack.Items.Clear();
                // Make only select folder possible.
                DisableAllControls();
                SelectFolderButton.IsEnabled = true;

                LoadImage(null);
            }
        }
        #endregion

        #region UI-Control-Management
        /// <summary>
        /// Focuses and enables <see cref="ResolutionBox"/>
        /// </summary>
        private void UseResolutionBox()
        {
            ResolutionBox.Focusable = true;
            ResolutionBox.Focus();
        }

        /// <summary>
        /// Unfocuses <see cref="ResolutionBox"/> 
        /// </summary>
        private void UnuseResolutionBox()
        {
            ResolutionBox.Focusable = false;
            Keyboard.ClearFocus();
            Focus();
        }

        /// <summary>
        /// Gives back, whether any folder is visible or not
        /// </summary>
        public bool IsAnyFolderVisible
        {
            get
            {
                // Get every item in the list
                foreach (ListBoxItem item in FoldersStack.Items)
                {
                    // if one of them turns out to be visible, then return true
                    if (item.Visibility == Visibility.Visible)
                        return true;
                }
                // Else false
                return false;
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
            GoBackButton.IsEnabled = true;
            DeleteImageButton.IsEnabled = true;
            RenameImageButton.IsEnabled = true;
            if (FoldersStack.Items.Count > 1)
            {
                DeleteFolderButton.IsEnabled = true;
            }
        }

        /// <summary>
        /// Disables all the controls beside the <see cref="SelectFolderButton"/>
        /// </summary>
        public void DisableControls()
        {
            SkipFileButton.IsEnabled = false;
            MoveFolderButton.IsEnabled = false;
            NewFolderButton.IsEnabled = false;
            GoBackButton.IsEnabled = false;
            DeleteImageButton.IsEnabled = false;
            DeleteFolderButton.IsEnabled = false;
            RenameImageButton.IsEnabled = false;
            // Don't need to disable EnterFolderButton, because there will always be folders to enter
        }

        /// <summary>
        /// Enable all controls. Literally all.
        /// </summary>
        public void EnableAllControls()
        {
            SkipFileButton.IsEnabled = true;
            MoveFolderButton.IsEnabled = true;
            NewFolderButton.IsEnabled = true;
            EnterFolderButton.IsEnabled = true;
            SelectFolderButton.IsEnabled = true;
            ResolutionBox.IsEnabled = true;
            GoBackButton.IsEnabled = true;
            DeleteImageButton.IsEnabled = true;
            RenameImageButton.IsEnabled = true;
            if (FoldersStack.Items.Count > 1)
            {
                DeleteFolderButton.IsEnabled = true;
            }
        }

        /// <summary>
        /// Disable all controls. Literally all.
        /// </summary>
        public void DisableAllControls()
        {
            SkipFileButton.IsEnabled = false;
            MoveFolderButton.IsEnabled = false;
            NewFolderButton.IsEnabled = false;
            EnterFolderButton.IsEnabled = false;
            SelectFolderButton.IsEnabled = false;
            ResolutionBox.IsEnabled = false;
            GoBackButton.IsEnabled = false;
            DeleteImageButton.IsEnabled = false;
            DeleteFolderButton.IsEnabled = false;
            RenameImageButton.IsEnabled = false;
        }

        /// <summary>
        /// Gets or sets whether the dark mode is supported
        /// </summary>
        public bool DarkMode
        {
            get
            {
                return Properties.Settings.Default.DarkModeEnabled;
            }

            set
            {
                // save the setting
                Properties.Settings.Default.DarkModeEnabled = value;
                Properties.Settings.Default.Save();

                // change the theme
                ThemeManager.ChangeAppStyle(System.Windows.Application.Current,
                    ThemeManager.DetectAppStyle().Item2,
                    ThemeManager.GetAppTheme(value ? "BaseDark" : "BaseLight"));

                // reflect the value on the button.
                DarkModeButton.IsChecked = value;

                // apply theme to seperator, as it isn't applied automatically.
                FolderImageSeperator.Background = value ? (Brush) FindResource("GrayBrush2") : (Brush) FindResource("GrayBrush6");
            }
        }

        /// <summary>
        /// Gets or sets the theme color of the app
        /// </summary>
        public static Color AccentColor
        {
            get
            {
                return (Color) ThemeManager.DetectAppStyle().Item2.Resources["AccentBaseColor"];
            }

            set
            {
                // create a new app style based on that color.
                ThemeManagerHelper.CreateAppStyleBy(value, true);
            }
        }

        /// <summary>
        /// Gets or sets whether the native accent color is used.
        /// </summary>
        public bool UseNativeColor
        {
            get { return (bool) GetValue(UseNativeColorProperty); }
            set { SetValue(UseNativeColorProperty, value); }
        }

        // Using a DependencyProperty as the backing store for UseNativeColor.  This enables animation, styling, binding, etc...
        public static readonly DependencyProperty UseNativeColorProperty =
            DependencyProperty.Register("UseNativeColor", typeof(bool), typeof(MainWindow), new PropertyMetadata(false, 
                (DependencyObject dp, DependencyPropertyChangedEventArgs e) =>
                {
                    // create a new accent based on the system accent color
                    if ((bool) e.NewValue)
                    {
                        try
                        {
                            // get theme from window
                            AccentColor =  NativeHelpers.GetWindowColorizationColor(false);
                        }
                        catch
                        {
                            Tuple<AppTheme, Accent> curStyle = ThemeManager.DetectAppStyle();
                            // fall back
                            ThemeManager.ChangeAppStyle(System.Windows.Application.Current,
                                ThemeManager.GetAccent("Amber"),
                                curStyle.Item1);
                        }
                    }
                    // reset the theme to the Amber accent color.
                    else
                        // use the default color.
                        ThemeManager.ChangeAppStyle(System.Windows.Application.Current,
                                                    ThemeManager.GetAccent("Amber"),
                                                    ThemeManager.DetectAppStyle().Item1);

                    // save the setting.
                    Properties.Settings.Default.UseNativeAccentColor = (bool) e.NewValue;
                    Properties.Settings.Default.Save();
                }));



        #endregion

        #region Image-Management
        /// <summary>
        /// Skips the current image and loads the next one
        /// </summary>
        public async void DoSkip()
        {
            if (SkipFileButton.IsEnabled == true)
            {
                //// set the preview image to nothing
                //PreviewImage.Source = null;
                // get the next image
                BitmapImage buffer = await imageManager.GetNextImage();
                // get the next path of the next image
                string path = imageManager.GetImagePath();

                // if the buffer is not null, load the image
                if (buffer != null)
                    LoadImage(buffer);
                // else disable the controls
                else
                {
                    // and unload it
                    LoadImage(null);
                    DisableControls();
                    GoBackButton.IsEnabled = true;
                }

                loadImageProgressSlider = false;
                ProgressSlider.Value = imageManager.CurrentIndex - 1;
            }
        }

        /// <summary>
        /// Moves the current image to the folder selected and loads the next one
        /// </summary>
        private async void DoMove()
        {
            if (folders.Count > 0)
            {
                if (MoveFolderButton.IsEnabled == true)
                {
                    //// set the preview image to nothing
                    //PreviewImage.Source = null;
                    // get the next path of the next image
                    string path = imageManager.GetImagePath();
                    // get the next image
                    BitmapImage buffer = await imageManager.GetNextImage();


                    // if the buffer is not null, load the image
                    if (buffer != null)
                        LoadImage(buffer);
                    // else disable the controls
                    else
                    {
                        // and unload it
                        LoadImage(null);
                        DisableControls();
                        GoBackButton.IsEnabled = true;
                    }

                    // Move the file
                    imageManager.MoveFileTo(path,
                        folders.ElementAt(FoldersStack.SelectedIndex) + "\\" +
                        System.IO.Path.GetFileName(path));

                    loadImageProgressSlider = false;
                    ProgressSlider.Value = imageManager.CurrentIndex - 1;
                }
            }
            else
            {
                System.Windows.MessageBox.Show("No Folders to move to. Create one first!", "Warning",
                    MessageBoxButton.OK, MessageBoxImage.Warning);
            }
        }

        /// <summary>
        /// Renames the image to the content of the <see cref="FileNameInfo"/>.
        /// </summary>
        public async void DoRename()
        {
            string currentImagePath = imageManager.GetImagePath();
            string newImageName = FileNameInfo.Text;
            string fileExtension = Path.GetExtension(currentImagePath);

            void loseFocus()
            {
                // remove focus, taken from decasteljaus anwer at https://stackoverflow.com/questions/2914495/wpf-how-to-programmatically-remove-focus-from-a-textbox
                // Move to a parent that can take focus
                FrameworkElement parent = (FrameworkElement) FileNameInfo.Parent;
                while (parent != null && parent is IInputElement && !((IInputElement) parent).Focusable)
                {
                    parent = (FrameworkElement) parent.Parent;
                }

                DependencyObject scope = FocusManager.GetFocusScope(FileNameInfo);
                FocusManager.SetFocusedElement(scope, parent as IInputElement);
            }

            if (newImageName != "")
            {
                // lose focus
                loseFocus();

                try
                {
                    if (Path.GetFileNameWithoutExtension(currentImagePath) != newImageName)
                        // rename the file.
                        imageManager.RenameFile(currentImagePath, $"{newImageName}{fileExtension}");
                }
                catch (Exception ex)
                {
                    await this.ShowMessageAsync(AppResources.CouldNotRenameFile.Replace("{file}", Path.GetFileName(imageManager.GetImagePath())),
                        AppResources.ExceptionMoreInfo.Replace("{message}", ex.Message));
                }
            }
            else
            {
                // lose focus
                loseFocus();
                FileNameInfo.Text = Path.GetFileNameWithoutExtension(currentImagePath);
            }
        }

        /// <summary>
        /// Asks the user if he wants to move the current image to the trash bin.
        /// </summary>
        public async void DoDelete()
        {
            if (DeleteImageButton.IsEnabled == true)
            {
                // get the current image to use it later.
                string currentImage = imageManager.GetImagePath();

                try
                {
                    // send image to the recycle bin
                    if (!FileOperationApiWrapper.Send(currentImage,
                        FileOperationApiWrapper.FileOperationFlags.FOF_ALLOWUNDO
                        | FileOperationApiWrapper.FileOperationFlags.FOF_WANTNUKEWARNING))
                    {
                        //// set the preview image to nothing
                        //PreviewImage.Source = null;
                        // get the next image
                        BitmapImage buffer = await imageManager.GetNextImage();
                        // get the next path of the next image
                        string path = imageManager.GetImagePath();

                        // if the buffer is not null, load the image
                        if (buffer != null)
                            LoadImage(buffer);
                        // else disable the controls
                        else
                        {
                            // and unload it
                            LoadImage(null);
                            DisableControls();
                            GoBackButton.IsEnabled = true;
                        }

                        loadImageProgressSlider = false;
                        ProgressSlider.Value = imageManager.CurrentIndex - 1;
                    }
                }
                catch (System.ComponentModel.Win32Exception ex)
                {
                    await this.ShowMessageAsync(
                        AppResources.CouldNotDeleteFile.Replace("{filename}", currentImage),
                        AppResources.CouldNotDeleteFileMessage.Replace("{filename}", currentImage)
                                                              .Replace("{errcode}", ex.NativeErrorCode.ToString())
                        );
                }
            }
        }

        /// <summary>
        /// Goes back to the last (skipped) image if possible.
        /// </summary>
        public async void GoBack()
        {
            // Actually go back in time!!!
            imageManager.GoBackImages();

            // Get the next image.
            LoadImage(await imageManager.GetNextImage());

            // Enable the controls again.
            EnableControls();

            loadImageProgressSlider = false;
            ProgressSlider.Value = imageManager.CurrentIndex - 1;
        }

        /// <summary>
        /// Load an image based on the value the <see cref="ProgressSlider"/> has.
        /// </summary>
        public async Task LoadImageFromSliderValue()
        {
            CurrentIndex = (int) ProgressSlider.Value;
            SkipFileButton.IsEnabled = true;
            // get the next image
            BitmapImage buffer = await imageManager.GetNextImage();
            // get the next path of the next image
            string path = imageManager.GetImagePath();

            // if the buffer is not null, load the image
            if (buffer != null)
                LoadImage(buffer);
            // else disable the controls
            else
            {
                // and unload it
                LoadImage(null);
                DisableControls();
                GoBackButton.IsEnabled = true;
            }
        }
        #endregion

        #region Performance
        /// <summary>
        /// Tells the garbage collector to collect garbage, reduces memory usage when called
        /// </summary>
        private void CollectGarbage()
        {
            GC.Collect();
            GC.WaitForPendingFinalizers();
        }
        #endregion

        #endregion




        #region Event Handlers
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
                // If the text of the search box it is not "" and alphabetic/numeric
                if (SearchBarBox.Text != "")
                {
                    // and if the item currently looped through doesn't contain the text of the search box
                    if (!foldersStackItem.Content.ToString().ToLower().Contains(SearchBarBox.Text.ToLower()))
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

            // Makes sure that an item is selected beforehand
            if (FoldersStack.SelectedItem != null)
                // Makes sure the item on top is selected,
                // when the currently selected one is not visible.
                if (((ListBoxItem) FoldersStack.SelectedItem).Visibility != Visibility.Visible)
                {
                    // Move selection to the second item (first is "..", which will not be focused anyway)
                    FoldersStack.SelectedIndex = 1;
                    // If the item is not visible, move down to the next one visible
                    if (((ListBoxItem) FoldersStack.SelectedItem).Visibility != Visibility.Visible)
                        MoveFolderSelectionDown();
                }

            // Makes sure currently selected element is visible at all times
            FoldersStack.ScrollToActiveItem();
        }

        /// <summary>
        /// When the user double clicks an element in the <see cref="FoldersStack"/>,
        /// then this method is called. It Enters the folder selected.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public void FolderStackItem_DoubleClick(object sender, EventArgs e)
        {
            if (EnterFolderButton.IsEnabled)
                EnterFolder();
        }

        ///// <summary>
        ///// Gets called, when the user clicks the "Select Folder" Button in
        ///// the tool bar
        ///// </summary>
        ///// <param name="sender">
        ///// the sender-object of the event
        ///// (used when generalizing the event for more controls)
        ///// </param>
        ///// <param name="e">Contains informations about the event</param>
        //private void SelectFolderButton_Click(object sender, RoutedEventArgs e)
        //{
        //    SelectFolder();
        //}

        ///// <summary>
        ///// Handles the Skip File buttons click event.
        ///// Loads next image when clicked, without performing an action
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void SkipFileButton_Click(object sender, RoutedEventArgs e)
        //{
        //    DoSkip();
        //}

        ///// <summary>
        ///// Called when the <see cref="MoveFolderButton.Click"/>-Event is being raised
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void MoveFolderButton_Click(object sender, RoutedEventArgs e)
        //{
        //    DoMove();
        //}

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
        public void FoldersStack_KeyDown(object sender, System.Windows.Input.KeyEventArgs e)
        {
            // Only check if the resolution box is not focused
            if (!ResolutionBox.Focusable)
                switch (e.Key)
                {
                    //        // When up key is pressed, move folder selection up
                    //        case Key.Up:
                    //            MoveFolderSelectionUp();
                    //                e.Handled = true;
                    //                break;

                    //        // When down key is pressed, move folder selection down
                    //        case Key.Down:
                    //            MoveFolderSelectionDown();
                    //                e.Handled = true;
                    //                break;

                    // Move the file when the right key has been pressed to the selected folder.
                    case Key.Right:
                        if (MoveFolderButton.IsEnabled && !SearchEnabled)
                        {
                            DoMove();
                            e.Handled = true;
                        }

                        break;

                    // Skips the file when the left key has been pressed or goes back one if it 
                    // was pressed with ctrl pressed
                    case Key.Left:
                        if (GoBackButton.IsEnabled &&
                            (Keyboard.IsKeyDown(Key.LeftCtrl) || Keyboard.IsKeyDown(Key.RightCtrl)))
                        {
                            GoBack();
                            e.Handled = true;
                        }
                        else if (SkipFileButton.IsEnabled && !SearchEnabled)
                        {
                            DoSkip();
                            e.Handled = true;
                        }
                        break;

                        //        // Opens Select Folder dialog
                        //        case Key.F2:
                        //            SelectFolder();
                        //                e.Handled = true;
                        //                break;

                        //        // Opens new folder Dialog
                        //        case Key.F3:
                        //            if (NewFolderButton.IsEnabled)
                        //                NewFolder();
                        //                e.Handled = true;
                        //                break;

                        //        // Opens dialog for resolution preference
                        //        case Key.F4:
                        //            if (ResolutionBox.IsEnabled)
                        //                UseResolutionBox();
                        //                e.Handled = true;
                        //                break;

                        //        // Opens the current image in the explorer
                        //        case Key.F5:
                        //            OpenImageInFileExplorer(folderSelector.GetImagePath());
                        //            break;

                        //        // "Enters" the folder
                        //        case Key.Enter:
                        //            if (IsAnyFolderVisible)
                        //                EnterFolder();
                        //                e.Handled = true;
                        //                break;

                        //        // Goes a folder upwards
                        //        case Key.Escape:
                        //            if (IsAnyFolderVisible)
                        //            {
                        //                FoldersStack.SelectedIndex = 0;
                        //                EnterFolder();
                        //            }
                        //                e.Handled = true;
                        //                break;
                        //        // For the keyboard-shortcut for opening the search
                        //        case Key.S:
                        //            if (Keyboard.IsKeyDown(Key.LeftCtrl) || Keyboard.IsKeyDown(Key.RightCtrl))
                        //                {
                        //                    e.Handled = true;
                        //                    SearchEnabled = !SearchEnabled;
                        //                }
                        //            break;
                }
        }

        ///// <summary>
        ///// Creates new folder when used
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void NewFolderButton_Click(object sender, RoutedEventArgs e)
        //{
        //    NewFolder();
        //}
        ///// <summary>
        ///// Enters the folder in question when called by user
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void EnterFolderButton_Click(object sender, RoutedEventArgs e)
        //{
        //    EnterFolder();
        //}

        ///// <summary>
        ///// Lets the user select a resolution, by that the images should be loaded.
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void SetHorizontalResolutionButton_Click(object sender, RoutedEventArgs e)
        //{
        //    SetResolution();
        //}

        /// <summary>
        /// Used for handling drag and drop, good for UX (obviously)
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private async void Window_Drop(object sender, System.Windows.DragEventArgs e)
        {
            if (e.Data.GetDataPresent(System.Windows.DataFormats.FileDrop, false))
            {
                // Note that you can have more than one file.
                string[] folder = (string[]) e.Data.GetData(System.Windows.DataFormats.FileDrop);

                // Only selects folder, if it exists.
                if (Directory.Exists(folder[0]))
                {
                    // Loads up the first folder (ignores the rest)
                    await SelectAndLoadFolder(folder[0]);

                    // Refresh folders
                    AddFoldersToFoldersStack();
                }
            }
        }

        /// <summary>
        /// Makes sure only numeric input gets entered
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ResolutionBox_PreviewTextInput(object sender, TextCompositionEventArgs e)
        {
            // if it is enabled,
            if (ResolutionBox.IsEnabled)
            {
                // get every character in the input
                foreach (char charItem in e.Text)
                {
                    // if it isn't a number, set he handled value to true to prevent input
                    if (!char.IsNumber(charItem))
                    {
                        e.Handled = true;
                    }
                }
            }
        }

        /// <summary>
        /// When enter has been pressed, unuse it
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ResolutionBox_KeyDown(object sender, System.Windows.Input.KeyEventArgs e)
        {
            // When enter was pressed
            if (e.Key == Key.Enter)
            {
                // If the resolution set is under 20, set it to 20
                if (!(int.Parse(ResolutionBox.Text) > 20))
                {
                    ResolutionBox.Text = 20.ToString();
                }

                // Unuse the resolution box
                UnuseResolutionBox();
                // and set the resolution to the max resolution
                MaxHorizontalResolution = int.Parse(ResolutionBox.Text);

                // Prevent further processing.
                e.Handled = true;
            }
        }

        /// <summary>
        /// Focuses <see cref="ResolutionBox"/> when clicking on the text box
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ResolutionBox_MouseDown(object sender, MouseButtonEventArgs e)
        {
            UseResolutionBox();
        }

        /// <summary>
        /// When the focus is lost, unuse it.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ResolutionBox_LostFocus(object sender, RoutedEventArgs e)
        {
            // If the resolution set is under 20, set it to 20
            if (!(int.Parse(ResolutionBox.Text) > 20))
            {
                ResolutionBox.Text = 20.ToString();
            }

            // Unuse the resolution box
            UnuseResolutionBox();
            // and set the resolution to the max resolution
            MaxHorizontalResolution = int.Parse(ResolutionBox.Text);
        }

        /// <summary>
        /// Event handling the clicking of the enable search button.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void EnableSearchButton_Click(object sender, RoutedEventArgs e)
        {
            SearchEnabled = (bool) EnableSearchButton.IsChecked;
        }

        ///// <summary>
        ///// Goes back in time (if the last image wasn't moved)
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void GoBackButton_Click(object sender, RoutedEventArgs e)
        //{
        //    GoBack();
        //}

        ///// <summary>
        ///// Called when hyperlinks meaned for opening something in File Explorer are clicked.
        ///// </summary>
        ///// <param name="sender"></param>
        ///// <param name="e"></param>
        //private void RequestOpeningInExplorer(object sender, System.Windows.Navigation.RequestNavigateEventArgs e)
        //{
        //    if (e.Uri != null && File.Exists(e.Uri.OriginalString))
        //        // Opens explorer.exe (File Explorer) and selects the given file.
        //        OpenImageInFileExplorer(e.Uri.OriginalString);
        //}

        #region CommandBindings
        /// <summary>
        /// Executed when the <see cref="Command.GoBackCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void GoBack_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (DialogsOpen <= 0 && GoBackButton.IsEnabled)
                GoBack();
        }

        /// <summary>
        /// Executed when the <see cref="Command.SelectFolderComand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SelectFolder_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            SelectFolder();
        }

        /// <summary>
        /// Executed when the <see cref="Command.CreateFolderCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void CreateFolder_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (NewFolderButton.IsEnabled)
                NewFolder();
        }

        /// <summary>
        /// Executed when the <see cref="Command.MoveUpCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MoveUp_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            MoveFolderSelectionUp();
        }

        /// <summary>
        /// Executed when the <see cref="Command.MoveDownCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MoveDown_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            MoveFolderSelectionDown();
        }

        /// <summary>
        /// Executed when the <see cref="Command.MoveImageCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MoveImage_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (DialogsOpen <= 0 && MoveFolderButton.IsEnabled && !SearchEnabled)
            {
                DoMove();
            }
        }

        /// <summary>
        /// Executed when the <see cref="Command.SkipImageCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void SkipImage_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (DialogsOpen <= 0 && SkipFileButton.IsEnabled && !SearchEnabled)
            {
                DoSkip();
            }
        }

        /// <summary>
        /// Executed when the <see cref="Command.FocusResolutionBoxCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void FocusResolutionBox_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (ResolutionBox.IsEnabled)
                UseResolutionBox();
        }

        /// <summary>
        /// Executed when the <see cref="Command.OpenInExplorerCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void OpenInExplorer_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (DialogsOpen <= 0 && OpenInExplorerLinkHost.Visibility == Visibility.Visible)
                OpenImageInFileExplorer(imageManager.GetImagePath());
        }

        /// <summary>
        /// Executed when the <see cref="Command.EnterFolderCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void EnterFolder_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            // prevent accidental entering.
            if (DialogsOpen <= 0)
            {
                // prevents entering a folder when a rename is initiated.
                if (FileNameInfo.IsFocused)
                {
                    DoRename();
                }
                else if (IsAnyFolderVisible && !ResolutionBox.Focusable)
                    EnterFolder();
            }
        }

        /// <summary>
        /// Executed when the <see cref="Command.EnterFolderCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void GoUpwards_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (DialogsOpen <= 0 && IsAnyFolderVisible)
            {
                // only go upwards when there is another parent folder.
                if (Path.GetPathRoot(imageManager.CurrentFolder) != imageManager.CurrentFolder)
                {
                    // select the top folder (..) and enter it, results in an upwards traversal in folders.
                    FoldersStack.SelectedIndex = 0;
                    EnterFolder();
                }
            }
        }

        /// <summary>
        /// Executed when the <see cref="Command.ToggleSearchCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ToggleSearch_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            SearchEnabled = !SearchEnabled;
        }

        /// <summary>
        /// Executed when the <see cref="Command.HelpCommand"/> is executed.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void Help_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            HelpButton.IsChecked = !HelpButton.IsChecked;
            if (HelpButton.IsChecked == true)
                helpWindow.Show();
            else
                helpWindow.Hide();
        }

        /// <summary>
        /// Called, when the user wants to delete the current image.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void DeleteCommand_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            DoDelete();
        }

        /// <summary>
        /// Called, when the user wants to rename the current image.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void RenameCommand_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            if (FileNameInfo.Focusable)
            {
                FileNameInfo.Focus();
                FileNameInfo.CaretIndex = FileNameInfo.Text.Length;
            }
        }

        /// <summary>
        /// Called, when the user wants to rename the current image.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void DeleteFolderCommand_Executed(object sender, ExecutedRoutedEventArgs e)
        {
            DeleteFolder();
        }
        #endregion

        /// <summary>
        /// Executed when the main Window closes. Saves the current dimensions of the window.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void Window_Closing(object sender, System.ComponentModel.CancelEventArgs e)
        {
#if !DEBUG_WINDOW
            if (WindowState == WindowState.Maximized)
            {
                // Use the RestoreBounds as the current values will be 0, 0 and the size of the screen
                Properties.Settings.Default.Top = RestoreBounds.Top;
                Properties.Settings.Default.Left = RestoreBounds.Left;
                Properties.Settings.Default.Height = RestoreBounds.Height;
                Properties.Settings.Default.Width = RestoreBounds.Width;
                Properties.Settings.Default.Maximized = true;
            }
            else
            {
                Properties.Settings.Default.Top = Top;
                Properties.Settings.Default.Left = Left;
                Properties.Settings.Default.Height = Height;
                Properties.Settings.Default.Width = Width;
                Properties.Settings.Default.Maximized = false;
            }

            Properties.Settings.Default.Save();
#endif

            // Close the help window.
            helpWindow.DoNotClose = false;
            helpWindow.Close();
        }

        /// <summary>
        /// Called when the progress slider was used. Loads the selected image.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ProgressSlider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (loadImageProgressSlider)
            {
                SliderValueChanged = true;
            }
            else
                loadImageProgressSlider = true;
        }

        /// <summary>
        /// Called when the window was loaded. Opens the <see cref="HelpWindow"/> when loaded.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private async void MetroWindow_Loaded(object sender, RoutedEventArgs e)
        {
            // set the datacontext to self.
            DataContext = this;

            // set the theme.
            DarkMode = Properties.Settings.Default.DarkModeEnabled;
            UseNativeColor = Properties.Settings.Default.UseNativeAccentColor;

            helpWindow = new HelpWindow();

            // register visiblity changed handler, to reflect changes in
            // helpWindow's visibility on the help toggle button.
            helpWindow.IsVisibleChanged += (o, e1) =>
                    HelpButton.IsChecked = ((Window) o).Visibility == Visibility.Visible;
#if !DEBUG_HELP
            if (Properties.Settings.Default.FirstRun)
            {
#endif
#if IS_UWP
                // on UWP use the accent color by default.
                UseNativeColor = true;
#endif

                // Open the help window
                helpWindow.Show();
                Properties.Settings.Default.FirstRun = false;
                Properties.Settings.Default.Save();
#if !DEBUG_HELP
            }
#endif

            // get the updater process from the App class.
            await Task.Run(async () =>
            {
                Process proc = App.updaterProcess;

                // if the process has been started, then...
                if (proc != null && proc.Start())
                    // use its standard output
                    using (StreamReader output = proc.StandardOutput)
                    {
                        // and stardard input
                        using (StreamWriter input = proc.StandardInput)
                        {
                            string changelog = "";
                            string version = "unknown";
                            string title = "";

                            // and read the output till the end.
                            while (!output.EndOfStream)
                            {
                                // get the last line from the stream.
                                string line = output.ReadLine();

                                // and if the updater asks for user consent, then
                                if (line == UpdaterConstants.UserConsent)
                                {
                                    await Dispatcher.InvokeAsync(async () =>
                                    {
                                        // Create a new UpdateDialog to ask for user consent.
                                        UpdateDialog dlg = new UpdateDialog() {
                                            ChangelogMarkdown = $"# {title}{Environment.NewLine}{changelog}",
                                            Version = version,
                                            Title = AppResources.UpdateConsentQuestion
                                        };
                                        // When the negative Button was clicked, then hide the metro dialog.
                                        dlg.NegativeButton.Click += (s, e1) =>
                                        {
                                            this.HideMetroDialogAsync(dlg);
                                            // signal that at least one dialog is not open anymore.
                                            DialogsOpen--;

                                            input.WriteLine(UpdaterConstants.Negative);
                                        };
                                        // When the positive button was clicked, then consent was given.
                                        dlg.PositiveButton.Click += (s, e1) =>
                                        {
                                            this.HideMetroDialogAsync(dlg);
                                            // signal that at least one dialog is not open anymore.
                                            DialogsOpen--;

                                            input.WriteLine(UpdaterConstants.Positive);
                                        };

                                        // signal that at least one dialog is open.
                                        DialogsOpen++;
                                        // show the dialog.
                                        await this.ShowMetroDialogAsync(dlg);
                                    });

                                    //// ask the user for consent and save the result.
                                    //MessageDialogResult result = await Dispatcher.Invoke(() => this.ShowMessageAsync("Update", AppResources.UpdateConsentQuestion,
                                    //    MessageDialogStyle.AffirmativeAndNegative, new MetroDialogSettings() {
                                    //        AffirmativeButtonText = AppResources.Yes,
                                    //        NegativeButtonText = AppResources.No,
                                    //        DefaultButtonFocus = MessageDialogResult.Affirmative
                                    //    }));
                                }
                                // Tell the user, that he has exceeded his rate limit.
                                else if (line == UpdaterConstants.RateLimitReached)
                                {
                                    if (output.ReadLine() == UpdaterConstants.ResetsOnTime)
                                        await Dispatcher.Invoke(() => this.ShowMessageAsync(AppResources.RateLimitExceeded,
                                            AppResources.RateLimitExceededMessage.Replace("{TIME}", output.ReadLine())));
                                }
                                // Get the Changelog from the updater
                                else if (line == UpdaterConstants.StartTransmittingChangelog)
                                {
                                    StringBuilder sb = new StringBuilder();
                                    // Read the first line
                                    string newLine = output.ReadLine();
                                    // while there is still a transmission, read the contents and
                                    // add them to the changelog
                                    while (newLine != UpdaterConstants.StopTransmittingChangelog)
                                    {
                                        sb.AppendLine(newLine);
                                        newLine = output.ReadLine();
                                    }
                                    // build the changelog and save it for later use.
                                    changelog = sb.ToString();
                                }
                                // if the version is going to be set, then read it.
                                else if (line == UpdaterConstants.UpdateVersion)
                                {
                                    version = output.ReadLine();
                                }
                                // if the title is going to be set, then read it.
                                else if (line == UpdaterConstants.UpdateTitle)
                                {
                                    title = output.ReadLine();
                                }
                                // Tell the user that an error occured, if it occured.
                                else if (line == UpdaterConstants.Error)
                                {
                                    await Dispatcher.Invoke(() => this.ShowMessageAsync(AppResources.UpdaterErrorOccured,
                                            $"{AppResources.UpdaterErrorOccuredMessage} {output.ReadLine()}"));
                                }
                            }
                        }
                    }
            });

            // Count the times the app was opened.
            if (Properties.Settings.Default.TimesAppOpenedSinceUpgrade < Math.Pow(2, sizeof(int) * 8))
            {
                Properties.Settings.Default.TimesAppOpenedSinceUpgrade++;
                Properties.Settings.Default.Save();
            }

#if DEBUG_SURVEY
            // reset the app counter when debugging the survey feature
            if (Properties.Settings.Default.TimesAppOpenedSinceUpgrade > 2)
            {
                Properties.Settings.Default.SystemServeyAnswered = false;
                Properties.Settings.Default.TimesAppOpenedSinceUpgrade = 0;
                Properties.Settings.Default.Save();
            }
#endif

#if (!IS_UWP)
            // When the survey on system informations wasn't yet answered, then ask the user if he would answer. 
            // Only ask on the second run and when not running on the UWP / Store version.
            if (!Properties.Settings.Default.SystemServeyAnswered
                && Properties.Settings.Default.TimesAppOpenedSinceUpgrade > 1)
            {
                if (await this.ShowMessageAsync($"{AppResources.CanYouHelpMe} {AppResources.ItsImportant}",
                                                AppResources.SystemInfoSurveyMessage,
                                                MessageDialogStyle.AffirmativeAndNegative,
                                                new MetroDialogSettings() {
                                                    AffirmativeButtonText = AppResources.Yes,
                                                    NegativeButtonText = AppResources.No,
                                                    DefaultButtonFocus = MessageDialogResult.Affirmative
                                                })
                    == MessageDialogResult.Affirmative)
                {
                    // if the user consents, then open the survey
                    Process.Start("https://docs.google.com/forms/d/e/1FAIpQLSerxGS4JVltRDUptS5BfRw5RK8hYG2WhGaT6jxwVuOMLw8lfg/viewform?usp=sf_link");
                }

                // Save that user answered the servey (or doesn't want to)
                Properties.Settings.Default.SystemServeyAnswered = true;
                Properties.Settings.Default.Save();
            }
#endif
        }

        /// <summary>
        /// Used, when the <see cref="HelpButton"/> was clicked. Toggles the HelpWindow on and off.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void HelpButton_Click(object sender, RoutedEventArgs e)
        {
            if (HelpButton.IsChecked == true)
                helpWindow.Show();
            else
                helpWindow.Hide();
        }

        /// <summary>
        /// Used, when the <see cref="FeedbackButton"/> was clicked. Opens Google Forms page.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void FeedbackButton_Click(object sender, RoutedEventArgs e)
        {
            //#if (!IS_UWP)
            //            System.Diagnostics.Process.Start("https://github.com/Lolle2000la/Image-Sort/issues/new");
            //#else
            //            System.Diagnostics.Process.Start("ms-windows-store://review/?productid=9PGDK9WN8HG6");
            //#endif
            Process.Start("https://docs.google.com/forms/d/e/1FAIpQLSeRLmo5uw0ZTqrgFAYqVE5Wyfthh_BeSCCG19FYmhADwiSRcw/viewform?usp=sf_link");
        }

        /// <summary>
        /// Handler used to prevent unwanted key-repitition.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MetroWindow_PreviewKeyDown(object sender, System.Windows.Input.KeyEventArgs e)
        {
            // if the event is called because the user hold down a key
            if (e.IsRepeat)
            {
                // and that key is either the left or right arrow key
                if (e.Key == Key.Left || e.Key == Key.Right)
                {
                    // and the focused control is not an textbox (where such behaviour is wished)
                    if (!typeof(System.Windows.Controls.TextBox).IsAssignableFrom(Keyboard.FocusedElement.GetType()))
                    {
                        // prevent repeat
                        e.Handled = true;
                    }
                }
            }
        }

        /// <summary>
        /// Used to initiate drag and drop.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void PreviewImage_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            InitializePreviewImageDrag();
        }

        /// <summary>
        /// Used to initiate drag and drop.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void PreviewImage_TouchDown(object sender, TouchEventArgs e)
        {
            InitializePreviewImageDrag();
        }

        /// <summary>
        /// Used to initiate drag and drop.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void PreviewImage_StylusDown(object sender, StylusDownEventArgs e)
        {
            InitializePreviewImageDrag();
        }

        /// <summary>
        /// Initializes drag for the preview image.
        /// </summary>
        private void InitializePreviewImageDrag()
        {
            // Create a new DataObject for dragging and set the Data
            System.Windows.DataObject dataObj = new System.Windows.DataObject();
            dataObj.SetImage((BitmapImage) PreviewImage.Source);
            dataObj.SetFileDropList(
                new System.Collections.Specialized.StringCollection() { imageManager.GetImagePath() });
            DragDrop.DoDragDrop(PreviewImage, dataObj, System.Windows.DragDropEffects.All);
        }

        /// <summary>
        /// Selects the hovered over element.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ListBoxItem_DragEnter(object sender, System.Windows.DragEventArgs e)
        {
            // only allow for drag and drop when there is t
            if (e.Data != null && ((System.Windows.DataObject) e.Data).ContainsImage()
                && ((System.Windows.DataObject) e.Data).ContainsFileDropList())
            {
                e.Effects = System.Windows.DragDropEffects.Move;
                FoldersStack.SelectedIndex = FoldersStack.Items.IndexOf(sender);
                ((UIElement) sender).AllowDrop = true;
                return;
            }
            ((UIElement) sender).AllowDrop = false;
        }

        /// <summary>
        /// Used to move an image into an folder.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ListBoxItem_Drop(object sender, System.Windows.DragEventArgs e)
        {
            if (MoveFolderButton.IsEnabled)
            {
                DoMove();
            }
        }

        /// <summary>
        /// Used to disable unwanted drops.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void MetroWindow_DragEnter(object sender, System.Windows.DragEventArgs e)
        {
            if (e.Data.GetDataPresent(System.Windows.DataFormats.FileDrop, false))
            {
                // Note that you can have more than one file. 
                string[] folder = (string[]) e.Data.GetData(System.Windows.DataFormats.FileDrop);

                // Only allow a folder, if it exists.
                if (Directory.Exists(folder[0]))
                {
                    AllowDrop = true;
                    return;
                }

                // If it isn't an folder, then just stop.
                AllowDrop = false;
            }
        }

        /// <summary>
        /// sets the dark mode to the value selected.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void DarkModeButton_Click(object sender, RoutedEventArgs e)
        {
            if (DarkModeButton.IsChecked.HasValue)
            {
                DarkMode = DarkModeButton.IsChecked.Value;
            }
        }
#endregion
    }

#region Commands
    public static class Command
    {
        /// <summary>
        /// Gives the user the option to select a(nother) folder, when executed.
        /// </summary>
        public static RoutedCommand SelectFolderCommand = new RoutedCommand();

        /// <summary>
        /// Gives the user the option to create a new folder, when executed.
        /// </summary>
        public static RoutedCommand CreateFolderCommand = new RoutedCommand();

        /// <summary>
        /// Moves the folder selection up, when executed.
        /// </summary>
        public static RoutedCommand MoveUpCommand = new RoutedCommand();

        /// <summary>
        /// Moves the folder selection down, when executed.
        /// </summary>
        public static RoutedCommand MoveDownCommand = new RoutedCommand();

        /// <summary>
        /// Moves the image to the currently selected folder, when executed.
        /// </summary>
        public static RoutedCommand MoveImageCommand = new RoutedCommand();

        /// <summary>
        /// Skips the current image, when executed.
        /// </summary>
        public static RoutedCommand SkipImageCommand = new RoutedCommand();

        /// <summary>
        /// Focuses the resolution box, when executed.
        /// </summary>
        public static RoutedCommand FocusResolutionBoxCommand = new RoutedCommand();

        /// <summary>
        /// Opens the current image in explorer, when executed.
        /// </summary>
        public static RoutedCommand OpenInExplorerCommand = new RoutedCommand();

        /// <summary>
        /// Enters the currently selected folder, when executed.
        /// </summary>
        public static RoutedCommand EnterFolderCommand = new RoutedCommand();

        /// <summary>
        /// Goes to the folder upwards in the hierarchy to the currently selected one, when executed.
        /// </summary>
        public static RoutedCommand GoUpwardsCommand = new RoutedCommand();

        /// <summary>
        /// Toggles the search box, when executed.
        /// </summary>
        public static RoutedCommand ToggleSearchCommand = new RoutedCommand();

        /// <summary>
        /// Goes back to the last image, when executed.
        /// </summary>
        public static RoutedCommand GoBackCommand = new RoutedCommand();

        /// <summary>
        /// Goes back to the last image, when executed.
        /// </summary>
        public static RoutedCommand HelpCommand = new RoutedCommand();

        /// <summary>
        /// Deletes the current image when executed.
        /// </summary>
        public static RoutedCommand DeleteCommand = new RoutedCommand();

        /// <summary>
        /// Focuses the <see cref="MainWindow.FileNameInfo"/> box to enable renaming when executed.
        /// </summary>
        public static RoutedCommand RenameCommand = new RoutedCommand();

        /// <summary>
        /// Deletes the currently selected folder when executed.
        /// </summary>
        public static RoutedCommand DeleteFolderCommand = new RoutedCommand();
    }
#endregion
}
