using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using System;
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

            if (MainWindow == null || App == null || Automation == null || CurrentPath == null) System.Diagnostics.Debug.WriteLine("Could not setup app fixture: One of the instances returned by setup is null");
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
