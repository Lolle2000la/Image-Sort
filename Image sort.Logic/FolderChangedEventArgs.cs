using System;

namespace Image_sort.Logic
{
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
