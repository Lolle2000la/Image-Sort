using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using System;
using System.Collections.Generic;
using System.Text;
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
    }
}
