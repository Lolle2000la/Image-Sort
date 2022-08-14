using System;
using System.IO;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Input;
using FlaUI.Core.Tools;
using FlaUI.Core.WindowsAPI;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    [Collection("App collection")]
    public class FolderActionsTests
    {
        private readonly Application app;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public FolderActionsTests(AppFixture appFixture)
        {
            (currentPath, app, _, mainWindow) = appFixture;
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

            Assert.True(Retry.WhileFalse(() => Directory.Exists(newFolderPath), timeout: TimeSpan.FromSeconds(5), interval: TimeSpan.FromMilliseconds(50)).Result);

            Assert.True(Directory.Exists(newFolderPath));

            // clean-up
            Directory.Delete(newFolderPath);
        }
    }
}