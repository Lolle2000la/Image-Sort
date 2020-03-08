using FlaUI.UIA3;
using FlaUI.Core;
using System;
using System.Collections.Generic;
using System.Text;
using Xunit;
using FlaUI.Core.AutomationElements;

namespace ImageSort.WPF.UiTests
{
    public class MoveTests : IDisposable
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public MoveTests()
        {
            (currentPath, app, automation, mainWindow) = SetupTeardownHelper.Setup();
        }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(currentPath, app, automation);
        }
    }
}
