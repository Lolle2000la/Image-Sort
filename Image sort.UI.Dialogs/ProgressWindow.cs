using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace Image_sort.UI.Dialogs
{
    /// <summary>
    /// Window for indicating, how long it will take to load the images in folder.
    /// </summary>
    public partial class ProgressWindow : Form
    {
        /// <summary>
        /// Constructor, creates the window.
        /// </summary>
        public ProgressWindow()
        {
            InitializeComponent();
        }

        /// <summary>
        /// Called when the Window has been loaded.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ProgressWindow_Load(object sender, EventArgs e)
        {
            
        }

        /// <summary>
        /// Event handler used for Routing up a progress changed event to the window directly
        /// </summary>
        /// <param name="sender">Object calling the method</param>
        /// <param name="e">Data given for the handler (like the actual progress)</param>
        public void LoadingProgress_Changed(object sender, ProgressChangedEventArgs e)
        {
            ChangeFileLoadingProgress(e.ProgressPercentage);
        }

        /// <summary>
        /// Sets the counter for files already loaded.
        /// </summary>
        /// <param name="currentFile">File currently loaded</param>
        /// <param name="minFiles">Minimum files that can be loaded. Most of the times = 0.</param>
        /// <param name="maxFiles">Maximum number of files that can be loaded.</param>
        public void ChangeFileProgress(int currentFile, int minFiles, int maxFiles)
        {
            // if the currentFile parameter doesn't match the given dimensions,
            // throw an ArgumentOutOfRangeException with the details.
            if (minFiles > currentFile || currentFile > maxFiles)
            {
                throw new ArgumentOutOfRangeException("currentFile", currentFile,
                    "The argument is bigger than the maxProgress or smaller than the " +
                    "minProgress given.");
            }

            // Sets the state of the progress bar indicating files loaded.
            pgrProgressPerFile.Minimum = minFiles;
            pgrProgressPerFile.Maximum = maxFiles;
            pgrProgressPerFile.Value = currentFile;

            // Sets the text of the label "lblProgressFiles",
            // so that the user can see how much files still are to be loaded.
            lblProgressFiles.Text = $"[{currentFile}/{maxFiles}]";
        }

        /// <summary>
        /// Sets the progress for the file being loaded,
        /// so that the user sees how long it will take to load it.
        /// </summary>
        /// <param name="currentProgress">The current progress the loading of the file has</param>
        /// <param name="minProgress">The minimum progress it can have. Default = 0</param>
        /// <param name="maxProgress">The maximum progress it can have. Default = 100</param>
        public void ChangeFileLoadingProgress(int currentProgress, int minProgress=0, int maxProgress=100)
        {
            // if the currentProgress parameter doesn't match the given dimensions,
            // throw an ArgumentOutOfRangeException with the details.
            if(maxProgress < currentProgress || currentProgress < minProgress)
            {
                throw new ArgumentOutOfRangeException("currentProgress", currentProgress,
                    "The argument is bigger than the maxProgress or smaller than the " +
                    "minProgress given.");
            }

            // Sets the state of the progress bar indicating files loaded.
            pgrProgressLoadingFile.Minimum = minProgress;
            pgrProgressLoadingFile.Maximum = maxProgress;
            pgrProgressLoadingFile.Value = currentProgress;

            // Sets the text of the label "lblProgressFiles",
            // so that the user can see how much files still are to be loaded.
            lblProgessFileLoad.Text = $"{currentProgress}%";
        }
    }
}
