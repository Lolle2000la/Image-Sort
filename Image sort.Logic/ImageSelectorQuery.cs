using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
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

        /************************************************************************/
        /*                                                                      */
        /* ATTRIBUTES                                                           */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Pool containing all the images in the folder
        /// </summary>
        private Queue<Image> imagePool = new Queue<Image>();
        /// <summary>
        /// Contains all the paths of all the images in the folder
        /// </summary>
        private Queue<string> imagePathPool = new Queue<string>();
        /// <summary>
        /// Contains the path to the current folder
        /// </summary>
        private string currentFolder;
        /// <summary>
        /// Contains the path to the current Image
        /// </summary>
        public string CurrentImage { get; set; }
        /// <summary>
        /// Defines the max resolution to be loaded 
        /// </summary>
        public int MaxHorizontalResolution { get; set; }











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
            imagePool.Clear();
            imagePathPool.Clear();
            CurrentImage = null;
            CollectGarbage();

            // Checks if the dir exists
            if (Directory.Exists(path))
            {
                // Sets the currentFolder var to path, so it can be easily retrieved
                currentFolder = path;

                // Gets images in the folder given in the parameter path
                IEnumerable<string> paths = Directory.EnumerateFiles(path, "*.*", 
                    SearchOption.TopDirectoryOnly)
                    .Where(s => s.EndsWith(".jpg") || s.EndsWith(".png")
                    || s.EndsWith(".gif") || s.EndsWith(".PNG") || s.EndsWith(".JPG")
                    || s.EndsWith(".GIF") || s.EndsWith(".tif") || s.EndsWith(".TIF")
                    || s.EndsWith(".tiff") || s.EndsWith(".TIFF"))/*.ToList<string>()*/;

                // goes through the image paths given and adds them to the image pool
                foreach (string currImagePath in paths)
                {
                    // Buffers image before putting it in the pool
                    Image image = new Image();
                    var uri = new Uri(currImagePath);
                    BitmapImage bitmapImage = new BitmapImage();

                    // Reads in the image into a bitmap for later usage, uses FileStream to ensure
                    // it works as it should by freeing the access to the file when unneeded
                    using (var stream =
                        new FileStream(currImagePath, FileMode.Open, FileAccess.Read, FileShare.Read))
                    {
                        bitmapImage.BeginInit();
                        bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
                        bitmapImage.DecodePixelWidth = MaxHorizontalResolution;
                        bitmapImage.StreamSource = stream;
                        bitmapImage.EndInit();
                    }

                    // Sets the source of the image and puts it into the queue/pool
                    image.Source = bitmapImage;
                    imagePool.Enqueue(image);
                    imagePathPool.Enqueue(uri.OriginalString);

                    // force frees unnecessary resources
                    //image.Source = null;
                    //image = null;
                    //uri = null;
                }


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
        /// Pulls the next <see cref="Image"/> out of the image pool
        /// </summary>
        /// <returns>returns the image as a <see cref="Image"/>, or <c>null</c> if no more images are in the folder</returns>
        public Image GetNextImage()
        {
            // Making sure, the image and string pool match up
            while (imagePool.Count > imagePathPool.Count)
            {
                imagePathPool.Dequeue();
            }

            // make sure everything works and there are images left in the queue
            if (imagePool.Count != 0)
            {
                // returns the image in queue
                return imagePool.Dequeue();
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
            // Making sure, the image and string pool match up
            while (imagePool.Count > imagePathPool.Count)
            {
                imagePathPool.Dequeue();
            }

            return imagePathPool.Dequeue();
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
    }
}
