﻿namespace Image_sort.UI.Dialogs
{
    partial class ProgressWindow
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(ProgressWindow));
            this.pgrProgressPerFile = new System.Windows.Forms.ProgressBar();
            this.lblProgressInfoText = new System.Windows.Forms.Label();
            this.lblProgressFiles = new System.Windows.Forms.Label();
            this.lblProgessFileLoad = new System.Windows.Forms.Label();
            this.lblProgressLoadingFilesText = new System.Windows.Forms.Label();
            this.pgrProgressLoadingFile = new System.Windows.Forms.ProgressBar();
            this.SuspendLayout();
            // 
            // pgrProgressPerFile
            // 
            this.pgrProgressPerFile.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.pgrProgressPerFile.Location = new System.Drawing.Point(12, 61);
            this.pgrProgressPerFile.Name = "pgrProgressPerFile";
            this.pgrProgressPerFile.Size = new System.Drawing.Size(773, 38);
            this.pgrProgressPerFile.TabIndex = 0;
            // 
            // lblProgressInfoText
            // 
            this.lblProgressInfoText.AutoSize = true;
            this.lblProgressInfoText.Location = new System.Drawing.Point(13, 13);
            this.lblProgressInfoText.Name = "lblProgressInfoText";
            this.lblProgressInfoText.Size = new System.Drawing.Size(141, 25);
            this.lblProgressInfoText.TabIndex = 1;
            this.lblProgressInfoText.Text = "Files to load: ";
            // 
            // lblProgressFiles
            // 
            this.lblProgressFiles.AutoSize = true;
            this.lblProgressFiles.Location = new System.Drawing.Point(161, 13);
            this.lblProgressFiles.Name = "lblProgressFiles";
            this.lblProgressFiles.Size = new System.Drawing.Size(0, 25);
            this.lblProgressFiles.TabIndex = 2;
            // 
            // lblProgessFileLoad
            // 
            this.lblProgessFileLoad.AutoSize = true;
            this.lblProgessFileLoad.Location = new System.Drawing.Point(250, 128);
            this.lblProgessFileLoad.Name = "lblProgessFileLoad";
            this.lblProgessFileLoad.Size = new System.Drawing.Size(0, 25);
            this.lblProgessFileLoad.TabIndex = 4;
            // 
            // lblProgressLoadingFilesText
            // 
            this.lblProgressLoadingFilesText.AutoSize = true;
            this.lblProgressLoadingFilesText.Location = new System.Drawing.Point(12, 128);
            this.lblProgressLoadingFilesText.Name = "lblProgressLoadingFilesText";
            this.lblProgressLoadingFilesText.Size = new System.Drawing.Size(232, 25);
            this.lblProgressLoadingFilesText.TabIndex = 3;
            this.lblProgressLoadingFilesText.Text = "Progress loading Files:";
            // 
            // pgrProgressLoadingFile
            // 
            this.pgrProgressLoadingFile.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.pgrProgressLoadingFile.Location = new System.Drawing.Point(12, 180);
            this.pgrProgressLoadingFile.Name = "pgrProgressLoadingFile";
            this.pgrProgressLoadingFile.Size = new System.Drawing.Size(773, 38);
            this.pgrProgressLoadingFile.TabIndex = 5;
            // 
            // ProgressWindow
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(12F, 25F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(797, 237);
            this.ControlBox = false;
            this.Controls.Add(this.pgrProgressLoadingFile);
            this.Controls.Add(this.lblProgessFileLoad);
            this.Controls.Add(this.lblProgressLoadingFilesText);
            this.Controls.Add(this.lblProgressFiles);
            this.Controls.Add(this.lblProgressInfoText);
            this.Controls.Add(this.pgrProgressPerFile);
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MaximizeBox = false;
            this.MinimizeBox = false;
            this.Name = "ProgressWindow";
            this.Text = "ProgressWindow";
            this.Load += new System.EventHandler(this.ProgressWindow_Load);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.ProgressBar pgrProgressPerFile;
        private System.Windows.Forms.Label lblProgressInfoText;
        private System.Windows.Forms.Label lblProgressFiles;
        private System.Windows.Forms.Label lblProgessFileLoad;
        private System.Windows.Forms.Label lblProgressLoadingFilesText;
        private System.Windows.Forms.ProgressBar pgrProgressLoadingFile;
    }
}