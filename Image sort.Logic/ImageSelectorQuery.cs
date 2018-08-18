using Image_sort.UI.Dialogs;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Forms;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Image_sort.Logic
{
    /// <summary>
    /// Gives back all the images out of the selected folder returns them one by one
    /// </summary>
    class ImageSelectorQuery
    {
        #region Attributes
        /************************************************************************/
        /*                                                                      */
        /* ATTRIBUTES                                                           */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Pool containing all the images in the folder
        /// </summary>
        private Queue<BitmapImage> imagePool = new Queue<BitmapImage>();
        /// <summary>
        /// Contains all the paths of all the images in the folder
        /// </summary>
        private List<string> imagePathPool = new List<string>();
        /// <summary>
        /// Contains the path to the current folder
        /// </summary>
        private string currentFolder;
        /// <summary>
        /// Contains the path to the current Image
        /// </summary>
        public string CurrentImage { get; private set; }
        /// <summary>
        /// Defines the max resolution to be loaded 
        /// </summary>
        public int MaxHorizontalResolution { get; set; }
        ///// <summary>
        ///// Window indicating the progress of the files being loaded to the user.
        ///// </summary>
        //private ProgressWindow progressWindow;
        /// <summary>
        /// Keeps track of which image we are at.
        /// </summary>
        public int CurrentIndex { get; set; } = 0;
        /// <summary>
        /// Contains all supported file types.
        /// </summary>
        public readonly string[] SupportedFileTypes = new string[] { ".jpg", ".png", ".gif", "tif", "tiff" };
        /// <summary>
        /// Handles the folder change event
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public delegate void FolderChangedHandler(object sender, FolderChangedEventArgs e);
        /// <summary>
        /// Raised when an folder(s content) was changed.
        /// </summary>
        public event FolderChangedHandler FolderChanged;
        /// <summary>
        /// Marks that a file is being moved.
        /// </summary>
        public bool MovingFile { get; set; } = false;
        /// <summary>
        /// Marks that a file is being renamed.
        /// </summary>
        public bool RenamingFile { get; set; } = false;
        #endregion




        #region Constructors
        /************************************************************************/
        /*                                                                      */
        /* CONSTRUCTORS                                                         */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// 
        /// </summary>
        public ImageSelectorQuery()
        {
            MaxHorizontalResolution = 1000;
        }

        public ImageSelectorQuery(int horizontalResolution)
        {
            MaxHorizontalResolution = horizontalResolution;
        }
        #endregion




        #region Methods
        /************************************************************************/
        /*                                                                      */
        /* METHODS                                                              */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Sets the Folder which should be used and prepares the image pool
        /// </summary>
        /// <param name="path">The path of the folder, of which all the images should get selected</param>
        /// <returns>
        /// returns true if it worked, false if it didn't 
        /// (for example because the folder does not exist)
        /// </returns>
        public bool SetCurrentFolderAsync(string path)
        {
            // Cleaning up in beforehand, to make sure everything works
            CleanUp();

            // Checks if the Directory exists
            if (Directory.Exists(path))
            {

                //// Sets a new instance for the progress window.
                //progressWindow = new ProgressWindow();

                // Sets the currentFolder var to path, so it can be easily retrieved
                currentFolder = path;

                // Gets images in the folder given in the parameter path
                IEnumerable<string> paths = Directory.EnumerateFiles(path, "*.*",
                    SearchOption.TopDirectoryOnly)
                    .Where(s => s.ToLower().EndsWithEither(SupportedFileTypes));


                //// Show the window.
                //progressWindow.Show();
                try
                {
                    //// set a few values to make sure the data is correct.
                    //progressWindow.ChangeFileProgress(0, 0, path.Count());


                    //// define an int holding the count of the file to load.
                    //int filesLoaded = 0;
                    
                    // goes through the image paths given and adds them to the image pool
                    foreach (string currImagePath in paths)
                    {
                        //// if the user wants to abort, throw an exception and abort.
                        //if (progressWindow.AbortRequested)
                        //{
                        //    throw new AbortException();
                        //}

                        // Buffers image before putting it in the pool
                        Uri uri = new Uri(currImagePath);

                        //BitmapImage buffer = await LoadImageAsync(currImagePath);
                        //if (buffer != null)
                        //{
                            // Sets the source of the image and puts it into the queue/pool
                            //imagePool.Enqueue(buffer);
                            imagePathPool.Add(uri.OriginalString);

                            // Sets the progress in the window for the files being loaded.
                            //progressWindow.ChangeFileProgress(++filesLoaded, 0, paths.Count());
                        //}
                    }
                }
                // if the loading was aborted, clean up anything and return false.
                catch (AbortException)
                {
                    // Clean up anything for the next start.
                    //CloseProgressWindow();
                    CleanUp();
                    return false;
                }
                //// Close progress window safely
                //CloseProgressWindow();

                // keeps track of changes in the current folder.
                FileSystemWatcher watcher = new FileSystemWatcher(path) {
                    EnableRaisingEvents = true
                };
                watcher.Renamed += OnFileRenamed;
                watcher.Deleted += OnFileDeleted;
                watcher.Created += OnFileCreated;


                // SUCCESS
                return true;
            }
            else
            {
                // set the current folder to null, to keep it from doing bad things
                currentFolder = null;

                // FAILURE
                return false;
            }
        }
                
        ///// <summary>
        ///// Closes the progress window.
        ///// </summary>
        //private void CloseProgressWindow()
        //{
        //    // Closes the window when it is no longer needed.
        //    while (!progressWindow.IsHandleCreated)
        //    {
        //        // Wait for the window to be created, before being closed.
        //        Task.Delay(1);
        //    }
        //    progressWindow.Close();
        //}

        /// <summary>
        /// Cleans up everything loaded. Clears the images and so on.
        /// </summary>
        private void CleanUp()
        {
            // Cleans up everything
            imagePool.Clear();
            imagePathPool.Clear();
            currentFolder = null;
            CurrentImage = null;
            CurrentIndex = 0;
            CollectGarbage();
        }

        /// <summary>
        /// Load the image at the path given. Returns null if it couldn't be loaded.
        /// </summary>
        /// <param name="path">Path to the image</param>
        /// <returns></returns>
        private async Task<BitmapImage> LoadImageAsync(string path)
        {
            // Holds the image
            BitmapImage bitmapImage/* = new BitmapImage()*/;

            // try loading the image
            try
            {
                bitmapImage = await Task.Run(() =>
                {
                    BitmapImage bitmap = new BitmapImage();
                    // Reads in the image into a bitmap for later usage, uses FileStream to ensure
                    // it works as it should by freeing the access to the file when unneeded
                    using (FileStream stream =
                        new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read))
                    {
                        // Loads the image
                        bitmap.BeginInit();
                        bitmap.CacheOption = BitmapCacheOption.OnLoad;
                        bitmap.DecodePixelWidth = MaxHorizontalResolution;
                        bitmap.StreamSource = stream;
                        bitmap.EndInit();
                    }

                    // Freeze bitmap to be able to use it from another thread.
                    if(bitmap.CanFreeze)
                        bitmap.Freeze();

                    // return the bitmap to the caller.
                    return bitmap;
                }).ConfigureAwait(true);
            }
            // If it isn't supported, tell the user which one is not
            catch (NotSupportedException)
            {
                // Show which one couldn't be opened.
                System.Windows.Forms.MessageBox.Show($"The image \"{Path.GetFileName(path)}\" could not be loaded.\n" +
                    $"It is not supported by this Program. Please make sure it is fully working");

                return null;
            }

            // return the bitmap image 
            return bitmapImage;
        }

        /// <summary>
        /// Pulls the next <see cref="Image"/> out of the image pool
        /// </summary>
        /// <returns>returns the image as a <see cref="Image"/>, or <c>null</c> if no more images are in the folder</returns>
        public async Task<BitmapImage> GetNextImage()
        {
            //// Making sure, the image and string pool match up
            //while (imagePool.Count > imagePathPool.Count)
            //{
            //    imagePathPool.Dequeue();
            //}

            // if there are no images left...
            if (CurrentIndex >= imagePathPool.Count)
            {
                // increment currentIndex by one for the "Go back" mechanism and return null.
                CurrentIndex++;
                return null;
            }

            // make sure everything works and there are images left in the queue
            if (imagePathPool.Count != 0)
            {
                // if the file doesn't exist, then try the next one
                if (!File.Exists(imagePathPool[CurrentIndex]))
                {
                    // increment currentIndex by one.
                    CurrentIndex++;
                    // Get the next image and return it.
                    return await GetNextImage();
                }
                else
                {
                    CurrentImage = imagePathPool[CurrentIndex];
                    CurrentIndex++;
                    // Buffer image and freeze it, so that it can be returned thread-safe.
                    BitmapImage bitmapImageBuffer = await LoadImageAsync(CurrentImage);
                    bitmapImageBuffer.Freeze();

                    // returns the image in queue
                    return bitmapImageBuffer;
                }
            }
            else
            {
                // FAILURE
                return null;
            }

            
        }

        /// <summary>
        /// Returns the path of the image
        /// </summary>
        /// <returns>Path to image</returns>
        public string GetImagePath()
        {
            //// Making sure, the image and string pool match up
            //while (imagePool.Count > imagePathPool.Count)
            //{
            //    imagePathPool.Dequeue();
            //}

            //if (imagePathPool.Count > 0)
            //    // SUCCESS
            //    return imagePathPool.Dequeue();
            //else
            //    // FAILURE
            //    return "";

            return CurrentImage;
        }

        /// <summary>
        /// Goes back in time (or, well, a list).
        /// </summary>
        /// <param name="amount">
        /// Specifies which amount of images it should be set back. 2 is the default, because that sets the image
        /// to the past one. 1 basically sets it to the current one, if that is needed to be loaded again.
        /// DO NOT USE NEGATIVE VALUES!
        /// </param>
        public void GoBackImages(int amount=2)
        {
            // Check if there is something to go back to
            if (imagePathPool.Count > 1 && CurrentIndex - amount >= 0)
                // If it just works, then just go back
                if (File.Exists(imagePathPool[CurrentIndex - amount]))
                {
                    CurrentIndex -= amount;
                }
                // else try to revert a move operation on the last image,
                // and if that is not possible, go back once more.
                else
                {
                    if (imagePathPool[CurrentIndex - amount].Contains("*"))
                    {
                        // The new and the old path have been seperated by a ":", because under Windows
                        // no path is allowed to contain ":". That makes it easy to seperate.
                        string[] paths = imagePathPool[CurrentIndex - amount].Split('*');
                        string oldPath = paths[0];
                        string newPath = paths[1];

                        if (File.Exists(newPath) && !File.Exists(oldPath))
                        {
                            try
                            {
                                File.Move(newPath, oldPath);
                                imagePathPool[CurrentIndex - amount] = oldPath;
                                CurrentIndex -= amount;
                                return;
                            }
                            // When access fails...
                            catch (IOException ex)
                            {
                                // Show the user a message box explaining why.
                                System.Windows.Forms.MessageBox.Show($"Could not move file. Error:\n\n{ex.Message}",
                                    "Error", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);
                            }

                        }
                        else
                        {
                            System.Windows.Forms.MessageBox.Show($"Could not move back the last image:" +
                                $"{System.Environment.NewLine}there already is a file named {Path.GetFileName(oldPath)}.",
                                "Error");
                        }
                    }
                    GoBackImages(amount + 1);
                }
            // if there is nothing to go back to, just go back to the current image.
            else
                CurrentIndex--;
        }

        /// <summary>
        /// Sets the resolution that should get targeted when loading
        /// </summary>
        /// <param name="horizontalResolution">Horizontal resolution targeted</param>
        public void SetResolution(int horizontalResolution)
        {
            MaxHorizontalResolution = horizontalResolution;
        }

        /// <summary>
        /// Tells the garbage collector to collect garbage, reduces memory usage when called
        /// </summary>
        private void CollectGarbage()
        {
            GC.Collect();
            GC.WaitForPendingFinalizers();
        }

        /// <summary>
        /// Appends the new location of the current image, so that it can be reverted to the 
        /// current path.
        /// </summary>
        /// <param name="newPath">The path to the new image location of the image.</param>
        public void AppendNewLocation(string newPath)
        {
            // Appends the new location to the image with a "*" (not allowed for paths/reserved)
            // so that it can be reverted again if needed.
            if (newPath.Length > 0 && File.Exists(newPath))
                imagePathPool[CurrentIndex - 2] += $"*{newPath}";
        }

        /// <summary>
        /// Returns the current progress in the pool
        /// </summary>
        /// <returns>(currentImage, maxImages)</returns>
        public (int, int) GetCurrentProgress()
        {
            return (CurrentIndex-1, imagePathPool.Count);
        }
        #endregion


        

        #region Event Handlers
        /// <summary>
        /// Keeps track of file renames.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        protected virtual void OnFileRenamed(object sender, RenamedEventArgs e)
        {
            // look if the renamed file is one of thhe used files that are currently used in Image sort.
            // if so, replace the old path known with the new one.
            for (int i = 0; i < imagePathPool.Count; i++)
            {
                if (imagePathPool[i] == e.OldFullPath)
                {
                    imagePathPool[i] = e.FullPath;
                }
            }

            // if the current image is the renamed one, then update the change.
            if (CurrentImage == e.OldFullPath)
            {
                CurrentImage = e.FullPath;
            }

            // only raise when there is no app initiated renaming in process.
            if (!RenamingFile)
                // raises the FolderChanged event
                FolderChanged(this, new FolderChangedEventArgs(e.FullPath));
        }

        /// <summary>
        /// Keeps track of any kind of file change relevant to the app.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        protected virtual void OnFileDeleted(object sender, FileSystemEventArgs e)
        {
            // if a file was deleted, remove it from the path pool.
            if (!MovingFile && e.ChangeType == WatcherChangeTypes.Deleted)
            {
                for (int i = 0; i < imagePathPool.Count; i++)
                {
                    if (imagePathPool[i] == e.FullPath)
                    {
                        imagePathPool.RemoveAt(i);
                    }
                }
            }

            // raises the FolderChanged event
            FolderChanged(this, new FolderChangedEventArgs(e.FullPath));
        }

        /// <summary>
        /// Used to keep track of file creations, like
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        protected virtual void OnFileCreated(object sender, FileSystemEventArgs e)
        {
            if (e.ChangeType == WatcherChangeTypes.Created)
            {
                // check if the created file is supported, and if it is, add it to the imagePathPool.
                if (e.FullPath.ToLower().EndsWithEither(SupportedFileTypes))
                {
                    imagePathPool.Add(e.FullPath);
                }
            }

            // raises the FolderChanged event
            FolderChanged(this, new FolderChangedEventArgs(e.FullPath));
        }
        #endregion
    }

    /// <summary>
    /// Contains helpers used in the <see cref="ImageSelectorQuery"/> class.
    /// </summary>
    static class ImageSelectoreQueryHelpers
    {
        /// <summary>
        /// Determines whether a <see cref="string"/> ends with one of the <see cref="string"/>s given in an 
        /// <see cref="Array"/> of <see cref="string"/>s.
        /// </summary>
        /// <param name="source"></param>
        /// <param name="endings"></param>
        /// <returns></returns>
        public static bool EndsWithEither(this string source, string[] endings)
        {
            bool endsWithOne = false;
            foreach (string ending in endings)
            {
                if (source.EndsWith(ending))
                {
                    endsWithOne = true;
                }
            }
            return endsWithOne;
        }
    }

    /// <summary>
    /// Contains info about the type of change.
    /// </summary>
    public class FolderChangedEventArgs : EventArgs
    {
        /// <summary>
        /// Contains the path to the changed file.
        /// </summary>
        public string ChangedFileFullPath { get; private set; }

        /// <summary>
        /// Initialises an new instance of <see cref="FolderChangedEventArgs"/>.
        /// </summary>
        /// <param name="changedFilePath">The full path to the changed file.</param>
        public FolderChangedEventArgs(string changedFilePath) : base()
        {
            ChangedFileFullPath = changedFilePath;
        }
    }
}
