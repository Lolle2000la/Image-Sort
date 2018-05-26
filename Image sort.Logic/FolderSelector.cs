using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Controls;
using System.Security.AccessControl;
using System.Windows.Media.Imaging;

namespace Image_sort.Logic
{
    /// <summary>
    /// Class for Selecting and Managing the folders selected
    /// </summary>
    public class FolderSelector
    {
        #region Atributes
        /************************************************************************/
        /*                                                                      */
        /* ATTRIBUTES                                                           */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Holds the path to the current folder selected
        /// </summary>
        private string CurrentFolderPath;

        /// <summary>
        /// Holds the instance of <see cref="ImageSelectorQuery"/> 
        /// </summary>
        private ImageSelectorQuery imageSelectorQuery;
        #endregion




        #region Constructors
        /************************************************************************/
        /*                                                                      */
        /* CONSTRUCTORS                                                         */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Creates a new <see cref="FolderSelector"/>.
        /// </summary>
        public FolderSelector()
        {
            imageSelectorQuery = new ImageSelectorQuery();
        }

        /// <summary>
        /// Creates a new <see cref="FolderSelector"/> with the given resolution.
        /// </summary>
        /// <param name="verticalResolution">
        /// Horizontal Resolution that the image should get loaded with.
        /// </param>
        public FolderSelector(int verticalResolution)
        {
            imageSelectorQuery = new ImageSelectorQuery(verticalResolution);
        }
        #endregion




        #region Methods
        /************************************************************************/
        /*                                                                      */
        /* METHODS                                                              */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Keeps track of which image we are at.
        /// </summary>
        public int CurrentIndex
        {
            get => imageSelectorQuery.CurrentIndex;
            set => imageSelectorQuery.CurrentIndex = value;
        }

        /// <summary>
        /// Selects Folder to use
        /// </summary>
        /// <param name="path">Path that should be returned</param>
        /// <returns>Returns true when successful and false when not</returns>
        public bool SelectAsync(string path)
        {
            // If the directory given exists, set the folder to that and return true
            if (Directory.Exists(path))
            {
                // Set the folder that should get processed.
                bool result = imageSelectorQuery.SetCurrentFolderAsync(path);

                if (result)
                    CurrentFolderPath = path;
                else
                    CurrentFolderPath = string.Empty;

                // Return the result of trying to select the folder.
                return result;
            }
            // if not, then set to null and return false
            CurrentFolderPath = string.Empty;
            return false;
        }

        /// <summary>
        /// <c>GetCurrentFolderPath</c> returns the current folder as a <see cref="string"/>,
        /// needed if it is in a sub folder
        /// </summary>
        /// <returns>Path to folder</returns>
        public string GetCurrentFolderPath()
        {
            return CurrentFolderPath;
        }

        /// <summary>
        /// Gives back current Image as <see cref="Image"/>
        /// </summary>
        /// <returns>Returns <see cref="BitmapImage"/></returns>
        public async Task<BitmapImage> GetNextImage()
        {
            return await imageSelectorQuery.GetNextImage();
        }

        /// <summary>
        /// Gives back the path to the image
        /// </summary>
        /// <returns>Path to the image</returns>
        public string GetImagePath()
        {
            return imageSelectorQuery.GetImagePath();
        }

        /// <summary>
        /// Moves the file from source to destination, makes sure it is unlocked
        /// throws IOException if image is not callable
        /// </summary>
        /// <param name="source">The <see cref="string"/> pointing to the source image</param>
        /// <param name="destination">The <see cref="string"/> pointing to it's destination</param>
        public void MoveFileTo(string source,string destination)
        {
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
                            while (File.Exists(GetPathWithNumber(destination, i)))
                            {
                                i++;
                                // Sets the path to the new path (for example: "image(2).jpg")
                                newDestinationName = GetPathWithNumber(destination, i);
                            }

                            // Sets the path to the new path (for example: "image(2).jpg")
                            newDestinationName = GetPathWithNumber(destination, i);

                            // Move the file
                            File.Move(source, newDestinationName);

                            // Remember the new location of the image
                            finalDestination = newDestinationName;
                        }
                    }

                    // Append the new location to the the current path for possible
                    // future retrievement and reversion.
                    imageSelectorQuery.AppendNewLocation(finalDestination);
                }
            }
            // When access fails...
            catch(IOException ex)
            {
                // Show the user a message box explaining why.
                System.Windows.Forms.MessageBox.Show($"Could not move file. Error:\n\n{ex.Message}",
                    "Error", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Error);
            }
        }

        /// <summary>
        /// Takes a Path as a <see cref="string"/> and gives it back with a number
        /// ("image.jpg" -> "image(i).jpg) of i
        /// </summary>
        /// <param name="original">The original string that should be used</param>
        /// <param name="i">The number which should get inserted</param>
        /// <returns></returns>
        public string GetPathWithNumber(string original, int i)
        {
            /* First get the directory, in which the original path lives in, 
             * then add the file name without extension at the end of it,
             * add the number between the (),
             * and finally add the extension back at it again. */
            return Path.GetDirectoryName(original) + @"\" +
                            Path.GetFileNameWithoutExtension(original) +
                            $"({i.ToString()})" +
                            Path.GetExtension(original);
        }

        /// <summary>
        /// Sets the resolution that should get targeted when loading
        /// </summary>
        /// <param name="horizontalResolution">Horizontal resolution targeted</param>
        public void SetResolution(int horizontalResolution)
        {
            imageSelectorQuery.SetResolution(horizontalResolution);
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
            imageSelectorQuery.GoBackImages(amount);
        }

        /// <summary>
        /// Returns the current progress in the pool
        /// </summary>
        /// <returns>(currentImage, maxImages)</returns>
        public (int, int) GetCurrentProgress()
        {
            return imageSelectorQuery.GetCurrentProgress();
        }
        #endregion
    }
}
