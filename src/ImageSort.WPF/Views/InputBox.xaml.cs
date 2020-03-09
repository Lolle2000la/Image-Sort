using AdonisUI.Controls;
using System;
using System.Windows;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for InputBox.xaml
    /// </summary>
    public partial class InputBox : AdonisWindow
    {
        public InputBox(string question, string title)
        {
            InitializeComponent();
            Question.Text = question;
            Title = title;
        }

        private void btnDialogOk_Click(object sender, RoutedEventArgs e)
        {
            this.DialogResult = true;
        }

        private void Window_ContentRendered(object sender, EventArgs e)
        {
            AnswerBox.SelectAll();
            AnswerBox.Focus();
        }

        public string Answer => AnswerBox.Text;
    }
}