using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;
using System.Windows.Media;

namespace Image_sort.UI.Dialogs
{
    /// <summary>
    /// Window for indicating, how long it will take to load the images in folder.
    /// </summary>
    public partial class ProgressWindow : Form
    {
        #region Attributes
        /// <summary>
        /// Contains whether the user wants to abort or not.
        /// </summary>
        private bool abortRequested = false;

        /// <summary>
        /// Returns if the user wants to abort or not.
        /// </summary>
        public bool AbortRequested
        {
            get
            {
                return abortRequested;
            }
        }
        #endregion




        #region Constructors
        /// <summary>
        /// Constructor, creates the window.
        /// </summary>
        public ProgressWindow()
        {
            InitializeComponent();
        }
        #endregion




        #region Methods
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
            Invoke(new MethodInvoker(delegate
            {
                // Sets the state of the progress bar indicating files loaded.
                pgrProgressPerFile.Minimum = minFiles;
                pgrProgressPerFile.Maximum = maxFiles;
                pgrProgressPerFile.Value = currentFile;

                // Sets the text of the label "lblProgressFiles",
                // so that the user can see how much files still are to be loaded.
                lblProgressFiles.Text = $"[{currentFile}/{maxFiles}]";

                // Force refresh window
                Refresh();
            }));
        }
        #endregion




        #region Event Handlers
        private void btnCancel_Click(object sender, EventArgs e)
        {
            // Showing that progress canceling is being made.
            btnCancel.Text = "Canceling";
            btnCancel.Enabled = false;

            // Signal that aborting is requested.
            abortRequested = true;
        }

        /// <summary>
        /// Called when the Window has been loaded.
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        private void ProgressWindow_Load(object sender, EventArgs e)
        {
            // Force refresh window
            this.Refresh();
        }
        #endregion
    }

    /// <summary>
    /// Exception for when the user presses the abort button.
    /// </summary>
    public class AbortException : Exception { }
}
