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
            var oldLocation = mainWindow.GetSelectedImage();
            var newLocation = Path.Combine(Directory.GetDirectories(currentPath)[0], Path.GetFileName(oldLocation));

            Assert.True(File.Exists(oldLocation));
            Assert.False(File.Exists(newLocation));

            app.WaitWhileBusy();

            mainWindow.Focus();

            // select folder
            Keyboard.Press(VirtualKeyShort.KEY_D);

            Keyboard.Press(VirtualKeyShort.KEY_S);

            app.WaitWhileBusy();

            var selectedImage = mainWindow.GetSelectedImage();

            // move image
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Move"))?.AsButton().Click();

            app.WaitWhileBusy();

            Assert.False(File.Exists(oldLocation));
            Assert.True(File.Exists(newLocation));

            // undo
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Undo"))?.AsButton().Click();

            app.WaitWhileBusy();

            // make sure the image is not added back twice, for example by the FileSystemWatcher in addition to the code itself
            Assert.Single(mainWindow.GetImages().Where(i => i == selectedImage));

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

        [Fact(DisplayName = "Can delete images")]
        public void CanDeleteImages()
        {
            var file = mainWindow.GetSelectedImage();

            Assert.True(File.Exists(file));

            app.WaitWhileBusy();

            mainWindow.Focus();

            // delete image
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Delete"))?.AsButton().Click();

            app.WaitWhileBusy();
            mainWindow.WaitUntilClickable();

            Assert.False(File.Exists(file));

            // clean-up
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Undo"))?.AsButton().Click();

            // make sure the image is not added back twice, for example by the FileSystemWatcher in addition to the code itself
            Assert.Single(mainWindow.GetImages().Where(i => i == file));

            Assert.True(File.Exists(file));
        }

        [Fact(DisplayName = "Can rename images")]
        public void CanRenameImages()
        {
            var file = mainWindow.GetSelectedImage();

            Assert.True(File.Exists(file));

            app.WaitWhileBusy();

            mainWindow.Focus();

            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Rename"))?.AsButton().Click();

            app.WaitWhileBusy();
            mainWindow.WaitUntilClickable();

            Keyboard.Type("mock renamed");
            Keyboard.Press(VirtualKeyShort.ENTER);

            app.WaitWhileBusy();
            mainWindow.WaitUntilClickable();

            Assert.False(File.Exists(file));
            Assert.Contains("mock renamed", Directory.EnumerateFiles(currentPath).Select(p => Path.GetFileNameWithoutExtension(p)));

            // clean-up
            mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Undo"))?.AsButton().Click();

            app.WaitWhileBusy();
            mainWindow.WaitUntilClickable();

            // make sure the image is not added back twice, for example by the FileSystemWatcher in addition to the code itself
            Assert.Single(mainWindow.GetImages().Where(i => i == file));

            Assert.True(File.Exists(file));
            Assert.DoesNotContain("mock renamed", Directory.EnumerateFiles(currentPath).Select(p => Path.GetFileNameWithoutExtension(p)));
        }
    }
}
