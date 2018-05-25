using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Controls;

namespace Image_sort.UI
{
    class ListBoxScroll : ListBox
    {
        public MainWindow MainWindowParent { get; set; }

        public ListBoxScroll() : base()
        {
            SelectionChanged += new SelectionChangedEventHandler(ListBoxScroll_SelectionChanged);
            PreviewKeyDown += PreventAccidentalHorizontalScrolling;
        }

        private void PreventAccidentalHorizontalScrolling(object sender, System.Windows.Input.KeyEventArgs e)
        {
            if (e.Key == System.Windows.Input.Key.Left || e.Key == System.Windows.Input.Key.Right)
            {
                MainWindowParent.FoldersStack_KeyDown(sender, e);
                e.Handled = true;
            }
        }

        void ListBoxScroll_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            ScrollToActiveItem();
        }

        public void ScrollToActiveItem()
        {
            ScrollIntoView(SelectedItem);
        }
    }
}
