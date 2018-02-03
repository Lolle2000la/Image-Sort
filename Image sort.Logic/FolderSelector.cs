using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Controls;
using System.Security.AccessControl;

namespace Image_sort.Logic
{
    /// <summary>
    /// Class for Selecting and Managing the folders selected
    /// </summary>
    public class FolderSelector
    {

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

        /// <summary>
        /// Counts the times access to a file failed 
        /// at the <see cref="MoveFileTo(string, string)"/> Method
        /// </summary>
        private int accessTimesFailed = 0;










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











        /************************************************************************/
        /*                                                                      */
        /* METHODS                                                              */
        /*                                                                      */
        /************************************************************************/

        /// <summary>
        /// Selects Folder to use
        /// </summary>
        /// <param name="path">Path that should be returned</param>
        /// <returns>Returns true when successful and false when not</returns>
        public bool Select(string path)
        {
            // If the directory given exists, set the folder to that and return true
            if (Directory.Exists(path))
            {
                CurrentFolderPath = path;
                imageSelectorQuery.SetCurrentFolder(path);
                return true;
            }
            // if not, then set to null and return false
            CurrentFolderPath = null;
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
        /// <returns>Returns <see cref="Image"/></returns>
        public Image GetNextImage()
        {
            return imageSelectorQuery.GetNextImage();
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
            // Counts the amount of times access failed
            
            try
            {
                // Actual moving operation
                File.Move(source, destination);
                accessTimesFailed = 0;
            }
            // When access fails...
            catch(IOException ex)
            {
                // ... and it failed 10 times ...
                if(accessTimesFailed < 10)
                {
                    throw ex; // ... throw back IOException back to caller
                }
                // ... or try again in 50 milliseconds when it did not
                Thread.Sleep(50);
                accessTimesFailed++;
                MoveFileTo(source, destination);
            }
        }

        /// <summary>
        /// Sets the resolution that should get targeted when loading
        /// </summary>
        /// <param name="horizontalResolution">Horizontal resolution targeted</param>
        public void SetResolution(int horizontalResolution)
        {
            imageSelectorQuery.SetResolution(horizontalResolution);
        }
    }
}
