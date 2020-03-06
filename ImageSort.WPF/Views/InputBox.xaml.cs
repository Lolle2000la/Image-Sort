using System;
using System.Collections.Generic;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Shapes;

namespace ImageSort.WPF.Views
{
    /// <summary>
    /// Interaction logic for InputBox.xaml
    /// </summary>
    public partial class InputBox : Window
    {
		public InputBox(string question, string title)
		{
			InitializeComponent();
			Question.Content = question;
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
