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
        /// Keeps track of which image we are at.
        /// </summary>
        private int currentIndex = 0;
        /// <summary>
        /// Contains the path to the current Image
        /// </summary>
        public string CurrentImage { get; private set; }
        /// <summary>
        /// Defines the max resolution to be loaded 
        /// </summary>
        public int MaxHorizontalResolution { get; set; }
        /// <summary>
        /// Window indicating the progress of the files being loaded to the user.
        /// </summary>
        private ProgressWindow progressWindow;
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
                    .Where(s => s.EndsWith(".jpg") || s.EndsWith(".png")
                    || s.EndsWith(".gif") || s.EndsWith(".PNG") || s.EndsWith(".JPG")
                    || s.EndsWith(".GIF") || s.EndsWith(".tif") || s.EndsWith(".TIF")
                    || s.EndsWith(".tiff") || s.EndsWith(".TIFF"))/*.ToList<string>()*/;


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
                        var uri = new Uri(currImagePath);

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
                
        /// <summary>
        /// Closes the progress window.
        /// </summary>
        private void CloseProgressWindow()
        {
            // Closes the window when it is no longer needed.
            while (!progressWindow.IsHandleCreated)
            {
                // Wait for the window to be created, before being closed.
                Task.Delay(1);
            }
            progressWindow.Close();
        }

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
            currentIndex = 0;
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
                    using (var stream =
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
            if (currentIndex >= imagePathPool.Count)
            {
                // increment currentIndex by one for the "Go back" mechanism and return null.
                currentIndex++;
                return null;
            }

            // make sure everything works and there are images left in the queue
            if (imagePathPool.Count != 0)
            {
                // if the file doesn't exist, then try the next one
                if (!File.Exists(imagePathPool[currentIndex]))
                {
                    // increment currentIndex by one.
                    currentIndex++;
                    // Get the next image and return it.
                    return await GetNextImage();
                }
                else
                {
                    CurrentImage = imagePathPool[currentIndex];
                    currentIndex++;
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
            if (imagePathPool.Count > 1 && currentIndex - amount >= 0)
                if (File.Exists(imagePathPool[currentIndex-amount]))
                {
                    currentIndex -= amount;
                }
                else
                {
                    GoBackImages(amount + 1);
                }
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
        #endregion
    }
}
