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
    public class ImageFolderManager : IImageManager
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
        /// Contains the path to the current Image
        /// </summary>
        public string CurrentImage { get; private set; }
        /// <summary>
        /// Defines the max resolution to be loaded 
        /// </summary>
        public int HorizontalResolution { get; set; }
        /// <summary>
        /// Keeps track of which image we are at.
        /// </summary>
        public int CurrentIndex { get; set; } = 0;
        /// <summary>
        /// Contains all supported file types.
        /// </summary>
        public readonly string[] SupportedFileTypes = new string[] { ".jpg", ".png", ".gif", "tif", "tiff" };
        /// <summary>
        /// Raised when an folder(s content) was changed.
        /// </summary>
        public event EventHandlerTypes.FolderChangedHandler FolderChanged;
        /// <summary>
        /// Marks that a file is being moved.
        /// </summary>
        public bool MovingFile { get; set; } = false;
        /// <summary>
        /// Marks that a file is being renamed.
        /// </summary>
        public bool RenamingFile { get; set; } = false;
        /// <summary>
        /// Holds the path to the current folder selected
        /// </summary>
        public string CurrentFolder { get; private set; }
        /// <summary>
        /// Contains an <see cref="List{T}"/> with all files, that are currently moving.
        /// </summary>
        private Dictionary<string, string> filesMoving = new Dictionary<string, string>();
        /// <summary>
        /// Contains an <see cref="IReadOnlyList{T}"/> with all files, that are currently moving.
        /// </summary>
        public IReadOnlyDictionary<string, string> FilesMoving => filesMoving;
        /// <summary>
        /// Keeps watch for changes on the file system.
        /// </summary>
        private FileSystemWatcher watcher;
        #endregion




        #region Constructors
        /************************************************************************/
        /*                                                                      */
        /* CONSTRUCTORS                                                         */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Constructs a new instance of ImageSelectorQuery.
        /// </summary>
        /// <remarks>The default for <see cref="HorizontalResolution"/> used will be 1000.</remarks>
        public ImageFolderManager()
        {
            HorizontalResolution = 1000;
        }
        /// <summary>
        /// Constructs a new instance of ImageSelectorQuery.
        /// </summary>
        /// <param name="horizontalResolution">
        /// The Value of <see cref="HorizontalResolution"/>, used to determine which horizontal resolution
        /// to load images with.
        /// </param>
        public ImageFolderManager(int horizontalResolution)
        {
            HorizontalResolution = horizontalResolution;
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
        public bool SetCurrentFolder(string path)
        {
            // Cleaning up in beforehand, to make sure everything works
            CleanUp();

            // Checks if the Directory exists
            if (Directory.Exists(path))
            {
                // Sets the CurrentFolderPath var to path, so it can be easily retrieved
                CurrentFolder = path;

                // Gets images in the folder given in the parameter path
                IEnumerable<string> paths = Directory.EnumerateFiles(path, "*.*",
                    SearchOption.TopDirectoryOnly)
                    .Where(s => s.ToLower().EndsWithEither(SupportedFileTypes));

                // goes through the image paths given and adds them to the image pool
                foreach (string currImagePath in paths)
                {
                    imagePathPool.Add(currImagePath);
                }

                // free the old watcher.
                if (watcher != null)
                {
                    watcher.EnableRaisingEvents = false;
                    watcher.Dispose();
                    watcher = null;
                }

                // keeps track of changes in the current folder.
                watcher = new FileSystemWatcher(path) {
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
                CurrentFolder = null;

                // FAILURE
                return false;
            }
        }

        /// <summary>
        /// Cleans up everything loaded. Clears the images and so on.
        /// </summary>
        private void CleanUp()
        {
            // Cleans up everything
            imagePool.Clear();
            imagePathPool.Clear();
            CurrentFolder = null;
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
                        bitmap.DecodePixelWidth = HorizontalResolution;
                        bitmap.StreamSource = stream;
                        bitmap.EndInit();
                    }

                    // Freeze bitmap to be able to use it from another thread.
                    if (bitmap.CanFreeze)
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
        public void GoBackImages(int amount = 2)
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
                                MovingFile = true;
                                MoveFileWithoutRemembering(newPath, oldPath);
                                imagePathPool[CurrentIndex - amount] = oldPath;
                                CurrentIndex -= amount;
                                MovingFile = false;
                                return;
                            }
                            // When access fails...
                            catch (IOException ex)
                            {
                                MovingFile = false;
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
            HorizontalResolution = horizontalResolution;
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
        public void RememberNewPathToImage(string newPath)
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
            return (CurrentIndex - 1, imagePathPool.Count);
        }

        /// <summary>
        /// Moves the file from source to destination, makes sure it is unlocked
        /// throws IOException if image is not callable
        /// </summary>
        /// <param name="source">The <see cref="string"/> pointing to the source image</param>
        /// <param name="destination">The <see cref="string"/> pointing to it's destination</param>
        public void MoveFileTo(string source, string destination)
        {
            MovingFile = true;
            try
            {
                // In the end "finalDestination" will contain the final destination,
                // which can be different from the one given.
                string finalDestination = "";
                // Only run, if there is an existing file, that has been given back.
                if (File.Exists(source))
                {
                    // Making sure the file doesn't already exist at the destination
                    if (!File.Exists(destination))
                    {
                        // signal that the file is moving
                        filesMoving.Add(source, destination);
                        // Actual moving operation
                        File.Move(source, destination);
                        // Remember the new location of the image
                        finalDestination = destination;
                    }
                    else
                    {
                        // Show the user a message box and ask him, if he wants to replace the image,
                        // rename it, or don't do anything.
                        System.Windows.Forms.DialogResult dialogResult = System.Windows.Forms.MessageBox.Show(
                            "The image does already exist in the selected folder." +
                            " Do you want to replace it?\n\n" +
                            "*No creates a new File for the image at the destination.", "Replace image?",
                            System.Windows.Forms.MessageBoxButtons.YesNoCancel,
                            System.Windows.Forms.MessageBoxIcon.Question);

                        // If the user wants to replace the image
                        if (dialogResult == System.Windows.Forms.DialogResult.Yes)
                        {
                            // signal that the file is moving
                            filesMoving.Add(source, destination);

                            // Delete the existing file and replace it
                            File.Delete(destination);
                            File.Move(source, destination);

                            // Remember the new location of the image
                            finalDestination = destination;
                        }
                        else if (dialogResult == System.Windows.Forms.DialogResult.No)
                        {
                            // Stores the number for the later renamed image (e.g. "image(i=2).jpg
                            int i = 0;

                            // Stores the path of the new destination name of the image to move.
                            string newDestinationName;

                            // increments i as long as it has to, so that the image to move
                            // can be moved with a name that doesn't exist yet.
                            while (File.Exists(ImageFolderManagerHelpers.GetPathWithNumber(destination, i)))
                            {
                                i++;
                                // Sets the path to the new path (for example: "image(2).jpg")
                                newDestinationName = ImageFolderManagerHelpers.GetPathWithNumber(destination, i);
                            }

                            // Sets the path to the new path (for example: "image(2).jpg")
                            newDestinationName = ImageFolderManagerHelpers.GetPathWithNumber(destination, i);

                            // signal that the file is moving
                            filesMoving.Add(source, newDestinationName);

                            // Move the file
                            File.Move(source, newDestinationName);

                            // Remember the new location of the image
                            finalDestination = newDestinationName;
                        }
                    }
                    
                    // Append the new location to the the current path for possible
                    // future retrievement and reversion.
                    RememberNewPathToImage(finalDestination);
                }
            }
            // When access fails...
            catch (IOException ex)
            {
                // Show the user a message box explaining why.
                System.Windows.Forms.MessageBox.Show($"Could not move file. Error:\n\n{ex.Message}",
                    "Error", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);
            }

            MovingFile = false;
        }

        /// <summary>
        /// 
        /// </summary>
        private void MoveFileWithoutRemembering(string source, string destination)
        {
            filesMoving.Add(source, destination);
            File.Move(source, destination);
        }

        /// <summary>
        /// Renames the given file and disables raising revents because of that.
        /// </summary>
        /// <param name="path">The path to the file to be renamed.</param>
        /// <param name="newName">the name of the file to be renamed.</param>
        public void RenameFile(string path, string newName)
        {
            // tell the imageSelectorQuery to not raise rename events.
            RenamingFile = true;

            // ensure the file exists
            if (!File.Exists(path))
            {
                throw new ArgumentException($"{path} does not exist.", "path");
            }
            // ensure the new name is not an directory.
            if (newName.Contains("\\") || newName.Contains("/"))
            {
                throw new ArgumentException($"{newName} contains an directory seperation char.", "newName");
            }
            // remame the file.
            File.Move(path, Path.Combine(Path.GetDirectoryName(path), newName));

            // tell the imageSelectorQuery to start raising rename events again.
            RenamingFile = false;
        }

        /// <summary>
        /// Gets whether an given path is currently moving or not.
        /// </summary>
        /// <param name="path">The path to be checked is moving.</param>
        /// <returns>Whether the path is moving or not.</returns>
        public async Task<bool> IsPathMovingAsync(string path)
        {
            bool pathMoving = false;

            await Task.Run(() =>
            {
                // get every path moving and compare
                Parallel.ForEach(filesMoving, (pair) =>
                {
                    string source = pair.Key;
                    string destination = pair.Value;

                    if (path == source || path == destination)
                    {
                        pathMoving = true;
                    }
                });
            });
            
            return pathMoving;
        }

        /// <summary>
        /// Removes an file given from the <see cref="filesMoving"/> based on key and value.
        /// </summary>
        /// <param name="path">the path to the file to be removed from <see cref="filesMoving"/>.</param>
        private void RemovePathFromMovingFiles(string path)
        {
            foreach (var item in filesMoving.Where(kvp => kvp.Value == path || kvp.Key == path).ToList())
            {
                filesMoving.Remove(item.Key);
            }
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

            if (!MovingFile)
                // raises the FolderChanged event
                FolderChanged(this, new FolderChangedEventArgs(e.FullPath));
        }

        /// <summary>
        /// Used to keep track of file creations, like
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        protected async virtual void OnFileCreated(object sender, FileSystemEventArgs e)
        {
            bool createdImageIsMoving = await IsPathMovingAsync(e.FullPath);

            if (!MovingFile && !createdImageIsMoving && e.ChangeType == WatcherChangeTypes.Created)
            {
                // check if the created file is supported, and if it is, add it to the imagePathPool.
                if (e.FullPath.ToLower().EndsWithEither(SupportedFileTypes))
                {
                    imagePathPool.Add(e.FullPath);

                    // raises the FolderChanged event
                    FolderChanged(this, new FolderChangedEventArgs(e.FullPath));
                }
            }
            else if (createdImageIsMoving)
            {
                RemovePathFromMovingFiles(e.FullPath);
            }
        }
        #endregion
    }
}
