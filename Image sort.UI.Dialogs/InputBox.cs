using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace Image_sort.UI.Dialogs
{


    public sealed class InputBox
    {
        /* Class mimics VB's InputBox function
         * Author: Scott Ketelaar
         * Developed: 4/27/2011
         */
        /// <summary>
        /// Provides input box, taken from 
        /// https://www.daniweb.com/programming/software-development/code/361925/pure-c-inputbox
        /// </summary>
        private class InputBoxForm : Form
        {
            
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
                this.cancelBtn = new System.Windows.Forms.Button();
                this.OKbtn = new System.Windows.Forms.Button();
                this.responseBox = new System.Windows.Forms.TextBox();
                this.promptLbl = new System.Windows.Forms.Label();
                this.SuspendLayout();
                // 
                // cancelBtn
                // 
                this.cancelBtn.DialogResult = System.Windows.Forms.DialogResult.Cancel;
                this.cancelBtn.Location = new System.Drawing.Point(291, 117);
                this.cancelBtn.Name = "cancelBtn";
                this.cancelBtn.Size = new System.Drawing.Size(75, 23);
                this.cancelBtn.TabIndex = 1;
                this.cancelBtn.Text = "Cancel";
                this.cancelBtn.UseVisualStyleBackColor = true;
                this.cancelBtn.Click += new System.EventHandler(this.button2_Click);
                // 
                // OKbtn
                // 
                this.OKbtn.Location = new System.Drawing.Point(210, 117);
                this.OKbtn.Name = "OKbtn";
                this.OKbtn.Size = new System.Drawing.Size(75, 23);
                this.OKbtn.TabIndex = 2;
                this.OKbtn.Text = "OK";
                this.OKbtn.UseVisualStyleBackColor = true;
                this.OKbtn.Click += new System.EventHandler(this.button1_Click);
                // 
                // responseBox
                // 
                this.responseBox.Location = new System.Drawing.Point(16, 91);
                this.responseBox.Name = "responseBox";
                this.responseBox.Size = new System.Drawing.Size(350, 20);
                this.responseBox.TabIndex = 4;
                // 
                // promptLbl
                // 
                this.promptLbl.Location = new System.Drawing.Point(13, 13);
                this.promptLbl.Name = "promptLbl";
                this.promptLbl.Size = new System.Drawing.Size(349, 75);
                this.promptLbl.TabIndex = 3;
                this.promptLbl.Text = "promptLbl";
                // 
                // Form1
                // 
                this.AcceptButton = this.OKbtn;
                this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
                this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
                this.CancelButton = this.cancelBtn;
                this.ClientSize = new System.Drawing.Size(374, 152);
                this.Controls.Add(this.responseBox);
                this.Controls.Add(this.promptLbl);
                this.Controls.Add(this.OKbtn);
                this.Controls.Add(this.cancelBtn);
                this.FormBorderStyle = System.Windows.Forms.FormBorderStyle.FixedDialog;
                this.MaximizeBox = false;
                this.MinimizeBox = false;
                this.Name = "Form1";
                this.ShowIcon = false;
                this.ShowInTaskbar = false;
                this.StartPosition = System.Windows.Forms.FormStartPosition.Manual;
                this.ResumeLayout(false);
                this.PerformLayout();
            }
            #endregion
            private System.Windows.Forms.Button cancelBtn;
            private System.Windows.Forms.Button OKbtn;
            private System.Windows.Forms.TextBox responseBox;
            private System.Windows.Forms.Label promptLbl;
            public InputBoxForm()
            {
                InitializeComponent();
            }
            public string Response { get { return this.responseBox.Text; } }
            private void button1_Click(object sender, EventArgs e)
            {
                this.DialogResult = DialogResult.OK;
                this.Close();
            }
            private void button2_Click(object sender, EventArgs e)
            {
                this.DialogResult = DialogResult.Cancel;
                this.responseBox.Text = "";
                this.Close();
            }
            public DialogResult ShowDialog(string Prompt, string Title, string DefaultResponse, int xPos, int yPos)
            {
                try
                {
                    this.promptLbl.Text = Prompt;
                    this.Text = Title;
                    this.responseBox.Text = DefaultResponse;
                    this.CenterToScreen();
                    this.Location = new System.Drawing.Point(
                        (xPos == -1 ? this.Location.X : xPos),
                        (yPos == -1 ? this.Location.Y : yPos));
                    return this.ShowDialog();
                }
                catch (Exception ex)
                {
                    throw ex;
                }
            }
        }
        /// <summary>
        /// Displays a prompt in a dialog box, waits for the user to input text or click a button, and then returns a string containing the contents of the text box.
        /// </summary>
        /// <param name="Prompt">String expression displayed as the message in the dialog box. The maximum length of Prompt is approximately 1024 characters, depending on the width of the characters used. If Prompt consists of more than one line, you can separate the lines using a carriage return character (\r), a linefeed character (\n), or a carriage returnâ€“linefeed character combination (\r\n) between each line.</param>
        /// <param name="Title">String expression displayed in the title bar of the dialog box.</param>
        /// <param name="DefaultResponse">String expression displayed in the text box as the default response if no other input is provided.</param>
        /// <param name="xPos">Numeric expression that specifies the left edge of the dialog box. If xPos is -1, the dialog box is centered horizontally.</param>
        /// <param name="yPos">Numeric expression that specifies the top edge of the dialog box. If yPos is -1, the dialog box is centered vertically.</param>
        /// <returns>A string containing the user's response.</returns>
        public static string Show(string Prompt, string Title="", string DefaultResponse="", int xPos=-1, int yPos=-1)
        {
            using (InputBoxForm ipb = new InputBoxForm())
            {
                ipb.ShowDialog(Prompt, Title, DefaultResponse, xPos, yPos);
                return ipb.Response;
            }
        }
    }


}
