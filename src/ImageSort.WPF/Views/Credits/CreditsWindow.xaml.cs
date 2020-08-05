using AdonisUI.Controls;

namespace ImageSort.WPF.Views.Credits
{
    /// <summary>
    /// Interaction logic for CreditsWindow.xaml
    /// </summary>
    public partial class CreditsWindow : AdonisWindow
    {
        private static CreditsWindow openWindow;

        public static CreditsWindow Window
        {
            get
            {
                if (openWindow == null)
                {
                    openWindow = new CreditsWindow();
                    openWindow.Closed += OnExistingWindowClosed;
                }

                return openWindow;
            }
        }

        private static void OnExistingWindowClosed(object sender, System.EventArgs e)
        {
            openWindow = null;
            (sender as CreditsWindow).Closed -= OnExistingWindowClosed;
        }

        private CreditsWindow()
        {
            InitializeComponent();
        }
    }
}
