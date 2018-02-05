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
        public ListBoxScroll() : base()
        {
            SelectionChanged += new SelectionChangedEventHandler(ListBoxScroll_SelectionChanged);
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
