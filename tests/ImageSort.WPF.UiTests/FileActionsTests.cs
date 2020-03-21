using FlaUI.UIA3;
using FlaUI.Core;
using System;
using System.Collections.Generic;
using System.Text;
using Xunit;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Tools;
using FlaUI.Core.Input;
using FlaUI.Core.WindowsAPI;
using System.Threading.Tasks;
using System.Linq;
using System.IO;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace ImageSort.WPF.UiTests
{
    [Collection("App collection")]
    public class FileActionsTests
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public FileActionsTests(AppFixture appFixture)
        {
            (currentPath, app, automation, mainWindow) = appFixture;
        }

        [Fact(DisplayName = "Can move image, undo and redo")]
        public void CanMoveImages()
        {
            var oldLocation = Directory.GetFiles(currentPath)[0];
            var newLocation = Path.Combine(Directory.GetDirectories(currentPath)[0], Path.GetFileName(oldLocation));

            Assert.True(File.Exists(oldLocation));
            Assert.False(File.Exists(newLocation));

            app.WaitWhileBusy();

            mainWindow.Focus();

            // select folder
            Keyboard.Press(VirtualKeyShort.KEY_D);

            Keyboard.Press(VirtualKeyShort.KEY_S);

            app.WaitWhileBusy();

            // move image
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Move"))?.AsButton().Click();

            app.WaitWhileBusy();

            Assert.False(File.Exists(oldLocation));
            Assert.True(File.Exists(newLocation));

            // undo
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Undo"))?.AsButton().Click();

            app.WaitWhileBusy();

            Assert.True(File.Exists(oldLocation));
            Assert.False(File.Exists(newLocation));

            // redo
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Redo"))?.AsButton().Click();

            app.WaitWhileBusy();

            Assert.False(File.Exists(oldLocation));
            Assert.True(File.Exists(newLocation));

            // clean-up
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Undo"))?.AsButton().Click();
        }
    }
}
