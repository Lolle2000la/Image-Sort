using FlaUI.UIA3;
using FlaUI.Core;
using System;
using System.Text;
using Xunit;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Input;
using FlaUI.Core.WindowsAPI;
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

            // select folder
            Keyboard.Press(VirtualKeyShort.KEY_D);

            Keyboard.Press(VirtualKeyShort.KEY_S);

            app.WaitWhileBusy();

            var selectedImage = mainWindow.GetSelectedImage();

            // move image
            mainWindow.ClickButton("Move");

            app.WaitWhileBusy();

            Assert.False(File.Exists(oldLocation));
            Assert.True(File.Exists(newLocation));

            // undo
            mainWindow.ClickButton("Undo");

            // make sure the image is not added back twice, for example by the FileSystemWatcher in addition to the code itself
            Assert.Single(mainWindow.GetImages().Where(i => i == selectedImage));

            Assert.True(File.Exists(oldLocation));
            Assert.False(File.Exists(newLocation));

            // redo
            mainWindow.ClickButton("Redo");

            Assert.False(File.Exists(oldLocation));
            Assert.True(File.Exists(newLocation));

            // clean-up
            mainWindow.ClickButton("Undo");

            // unselect folder
            Keyboard.Press(VirtualKeyShort.KEY_A);
            Keyboard.Press(VirtualKeyShort.KEY_A);
        }

        [Fact(DisplayName = "Can delete images")]
        public void CanDeleteImages()
        {
            var file = mainWindow.GetSelectedImage();

            Assert.True(File.Exists(file));

            // delete image
            mainWindow.ClickButton("Delete");

            Assert.False(File.Exists(file));

            // clean-up
            mainWindow.ClickButton("Undo");

            // make sure the image is not added back twice, for example by the FileSystemWatcher in addition to the code itself
            Assert.Single(mainWindow.GetImages().Where(i => i == file));

            Assert.True(File.Exists(file));
        }
    }
}
