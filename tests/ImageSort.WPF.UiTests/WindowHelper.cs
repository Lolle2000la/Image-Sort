using System.Collections.Generic;
using System.Linq;
using FlaUI.Core.AutomationElements;

namespace ImageSort.WPF.UiTests
{
    internal static class WindowHelper
    {
        private static ListBox GetImagesBox(this Window mainWindow)
        {
            return mainWindow
                .FindFirstDescendant(cf => cf.ByAutomationId("Images"))
                .FindFirstDescendant(cf => cf.ByAutomationId("Images")).AsListBox();
        }

        public static IEnumerable<string> GetImages(this Window mainWindow)
        {
            return mainWindow
                .GetImagesBox()
                .FindAllChildren()
                .Select(e => e.AsListBoxItem())
                .Select(e => e.Name);
        }

        public static string GetSelectedImage(this Window mainWindow)
        {
            return mainWindow
                .GetImagesBox()
                .SelectedItem.Name;
        }
    }
}