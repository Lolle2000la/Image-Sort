using System.Collections.Generic;
using System.Threading.Tasks;
using System.Windows.Media.Imaging;

namespace Image_sort.Logic
{
    /// <summary>
    /// Gives back all the images out of the selected folder returns them one by one
    /// </summary>
    public interface IImageManager
    {
        /// <summary>
        /// Contains the path to the current Image
        /// </summary>
        string CurrentImage { get; }
        /// <summary>
        /// Keeps track of which image we are at.
        /// </summary>
        int CurrentIndex { get; set; }
        /// <summary>
        /// Defines the max resolution to be loaded 
        /// </summary>
        int HorizontalResolution { get; set; }
        /// <summary>
        /// Marks that a file is being moved.
        /// </summary>
        bool MovingFile { get; set; }
        /// <summary>
        /// Marks that a file is being renamed.
        /// </summary>
        bool RenamingFile { get; set; }
        /// <summary>
        /// Holds the path to the current folder selected
        /// </summary>
        string CurrentFolder { get; }
        /// <summary>
        /// Contains an <see cref="IReadOnlyList{T}"/> with all files, that are currently moving.
        /// </summary>
        IReadOnlyDictionary<string, string> FilesMoving { get; }

        /// <summary>
        /// Raised when an folder(s content) was changed.
        /// </summary>
        event EventHandlerTypes.FolderChangedHandler FolderChanged;

        /// <summary>
        /// Appends the new location of the current image, so that it can be reverted to the 
        /// current path.
        /// </summary>
        /// <param name="newPath">The path to the new image location of the image.</param>
        void RememberNewPathToImage(string newPath);
        /// <summary>
        /// Returns the current progress in the pool
        /// </summary>
        /// <returns>(currentImage, maxImages)</returns>
        (int, int) GetCurrentProgress();
        /// <summary>
        /// Returns the path of the image
        /// </summary>
        /// <returns>Path to image</returns>
        string GetImagePath();
        /// <summary>
        /// Pulls the next <see cref="BitmapImage"/> out of the image pool
        /// </summary>
        /// <returns>returns the image as a <see cref="BitmapImage"/>, or <c>null</c> if no more images are in the folder</returns>
        Task<BitmapImage> GetNextImage();
        /// <summary>
        /// Goes back in time (or, well, a list).
        /// </summary>
        /// <param name="amount">
        /// Specifies which amount of images it should be set back. 2 is the default, because that sets the image
        /// to the past one. 1 basically sets it to the current one, if that is needed to be loaded again.
        /// DO NOT USE NEGATIVE VALUES!
        /// </param>
        void GoBackImages(int amount = 2);
        /// <summary>
        /// Sets the Folder which should be used and prepares the image pool
        /// </summary>
        /// <param name="path">The path of the folder, of which all the images should get selected</param>
        /// <returns>
        /// returns true if it worked, false if it didn't 
        /// (for example because the folder does not exist)
        /// </returns>
        bool SetCurrentFolder(string path);
        /// <summary>
        /// Sets the resolution that should get targeted when loading
        /// </summary>
        /// <param name="horizontalResolution">Horizontal resolution targeted</param>
        void SetResolution(int horizontalResolution);
        /// <summary>
        /// Moves the file from source to destination, makes sure it is unlocked
        /// throws IOException if image is not callable
        /// </summary>
        /// <param name="source">The <see cref="string"/> pointing to the source image</param>
        /// <param name="destination">The <see cref="string"/> pointing to it's destination</param>
        void MoveFileTo(string source, string destination);
        /// <summary>
        /// Renames the given file and disables raising revents because of that.
        /// </summary>
        /// <param name="path">The path to the file to be renamed.</param>
        /// <param name="newName">the name of the file to be renamed.</param>
        void RenameFile(string path, string newName);
        /// <summary>
        /// Gets whether an given path is currently moving or not.
        /// </summary>
        /// <param name="path">The path to be checked is moving.</param>
        /// <returns>Whether the path is moving or not.</returns>
        Task<bool> IsPathMovingAsync(string path);
    }
}