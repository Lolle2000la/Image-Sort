using FlaUI.Core;
using FlaUI.Core.AutomationElements;

namespace ImageSort.WPF.UiTests;

internal static class ControlHelper
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