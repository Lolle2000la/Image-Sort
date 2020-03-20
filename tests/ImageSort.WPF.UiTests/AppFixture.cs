using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using System;
using System.Collections.Generic;
using System.Text;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    public class AppFixture : IDisposable
    {
        private string CurrentPath { get; }
        public Application App { get; }
        public UIA3Automation Automation { get; }
        public Window MainWindow { get; }

        public AppFixture()
        {
            (CurrentPath, App, Automation, MainWindow) = SetupTeardownHelper.Setup();
        }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(CurrentPath, App, Automation);
        }

        internal void Deconstruct(out string currentPath, out Application app, out UIA3Automation automation, out Window mainWindow)
        {
            currentPath = CurrentPath;
            app = App;
            automation = Automation;
            mainWindow = MainWindow;
        }
    }

    [CollectionDefinition("App collection", DisableParallelization = true)]
    public class AppCollection : ICollectionFixture<AppFixture>
    {

    }
}
