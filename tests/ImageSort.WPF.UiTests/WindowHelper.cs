using FlaUI.Core.AutomationElements;
using System.Collections.Generic;
using System.Linq;

namespace ImageSort.WPF.UiTests
{
    static class WindowHelper
    {
        private static ListBox GetImagesBox(this Window mainWindow) => mainWindow
            .FindFirstDescendant(cf => cf.ByAutomationId("Images")).FindFirstDescendant(cf => cf.ByAutomationId("Images")).AsListBox();

        public static IEnumerable<string> GetImages(this Window mainWindow) => mainWindow
            .GetImagesBox()
            .FindAllChildren()
            .Select(e => e.AsListBoxItem())
            .Select(e => e.Name);

        public static string GetSelectedImage(this Window mainWindow) => mainWindow
            .GetImagesBox()
            .SelectedItem.Name;
    }
}
