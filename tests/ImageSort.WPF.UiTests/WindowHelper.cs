using FlaUI.Core.AutomationElements;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace ImageSort.WPF.UiTests
{
    static class WindowHelper
    {
        public static IEnumerable<string> GetImages(this Window mainWindow) => mainWindow
            .FindFirstDescendant(cf => cf.ByAutomationId("Images")).FindFirstDescendant(cf => cf.ByAutomationId("Images")).AsListBox()
            .FindAllChildren()
            .Select(e => e.AsListBoxItem())
            .Select(e => e.Name);
    }
}
