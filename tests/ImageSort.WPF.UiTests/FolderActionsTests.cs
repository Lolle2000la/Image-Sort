using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Input;
using FlaUI.Core.WindowsAPI;
using FlaUI.UIA3;
using System;
using System.IO;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    [Collection("App collection")]
    public class FolderActionsTests
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public FolderActionsTests(AppFixture appFixture)
        {
            (currentPath, app, automation, mainWindow) = appFixture;
        }

        [Fact(DisplayName = "Can create folders and reacts to its deletion")]
        public void CanCreateFolders()
        {
            const string newFolderName = "new folder";
            var newFolderPath = Path.Combine(currentPath, newFolderName);

            mainWindow.ClickButton("CreateFolder");

            Keyboard.Type(newFolderName);
            Keyboard.Press(VirtualKeyShort.ENTER);

            app.WaitWhileBusy();
            mainWindow.WaitUntilClickable();

            Assert.True(Directory.Exists(newFolderPath));

            // clean-up
            Directory.Delete(newFolderPath);
        }
    }
}
