namespace Image_sort.UI.Dialogs
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
            this.btnCancel = new System.Windows.Forms.Button();
            this.SuspendLayout();
            // 
            // pgrProgressPerFile
            // 
            this.pgrProgressPerFile.Location = new System.Drawing.Point(12, 61);
            this.pgrProgressPerFile.Margin = new System.Windows.Forms.Padding(10);
            this.pgrProgressPerFile.Name = "pgrProgressPerFile";
            this.pgrProgressPerFile.Size = new System.Drawing.Size(770, 40);
            this.pgrProgressPerFile.TabIndex = 0;
            // 
            // lblProgressInfoText
            // 
            this.lblProgressInfoText.AutoSize = true;
            this.lblProgressInfoText.Location = new System.Drawing.Point(7, 19);
            this.lblProgressInfoText.Margin = new System.Windows.Forms.Padding(10);
            this.lblProgressInfoText.Name = "lblProgressInfoText";
            this.lblProgressInfoText.Size = new System.Drawing.Size(141, 25);
            this.lblProgressInfoText.TabIndex = 1;
            this.lblProgressInfoText.Text = "Files to load: ";
            // 
            // lblProgressFiles
            // 
            this.lblProgressFiles.AutoSize = true;
            this.lblProgressFiles.Location = new System.Drawing.Point(155, 19);
            this.lblProgressFiles.Name = "lblProgressFiles";
            this.lblProgressFiles.Size = new System.Drawing.Size(0, 25);
            this.lblProgressFiles.TabIndex = 2;
            // 
            // btnCancel
            // 
            this.btnCancel.AutoSize = true;
            this.btnCancel.Location = new System.Drawing.Point(669, 113);
            this.btnCancel.Margin = new System.Windows.Forms.Padding(10);
            this.btnCancel.Name = "btnCancel";
            this.btnCancel.Size = new System.Drawing.Size(113, 40);
            this.btnCancel.TabIndex = 1;
            this.btnCancel.Text = "Cancel";
            this.btnCancel.UseVisualStyleBackColor = true;
            this.btnCancel.Click += new System.EventHandler(this.btnCancel_Click);
            // 
            // ProgressWindow
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(12F, 25F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.AutoSize = true;
            this.ClientSize = new System.Drawing.Size(794, 167);
            this.ControlBox = false;
            this.Controls.Add(this.btnCancel);
            this.Controls.Add(this.lblProgressFiles);
            this.Controls.Add(this.lblProgressInfoText);
            this.Controls.Add(this.pgrProgressPerFile);
            this.DoubleBuffered = true;
            this.FormBorderStyle = System.Windows.Forms.FormBorderStyle.FixedDialog;
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MaximizeBox = false;
            this.MinimizeBox = false;
            this.Name = "ProgressWindow";
            this.Text = "Loading...";
            this.Load += new System.EventHandler(this.ProgressWindow_Load);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        #endregion

        private System.Windows.Forms.ProgressBar pgrProgressPerFile;
        private System.Windows.Forms.Label lblProgressInfoText;
        private System.Windows.Forms.Label lblProgressFiles;
        private System.Windows.Forms.Button btnCancel;
    }
}