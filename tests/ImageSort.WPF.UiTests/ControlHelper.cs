using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using System.Text;

namespace ImageSort.WPF.UiTests
{
    static class ControlHelper
    {
        public static Application App { get; set; }
        public static Window MainWindow { get; set; }

        public static void ClickButton(this AutomationElement element, string automationId)
        {
            element.FindFirstDescendant(cf => cf.ByAutomationId(automationId))?.AsButton().Click();

            App.WaitWhileBusy();
            MainWindow.WaitUntilClickable();
        }
    }
}
