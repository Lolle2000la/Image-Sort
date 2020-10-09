using System;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using Debug = System.Diagnostics.Debug;

namespace ImageSort.WPF.UiTests
{
    public class AppFixture : IDisposable
    {
        public AppFixture()
        {
            (CurrentPath, App, Automation, MainWindow) = SetupTeardownHelper.Setup();

            if (MainWindow == null || App == null || Automation == null || CurrentPath == null)
                Debug.WriteLine("Could not setup app fixture: One of the instances returned by setup is null");
        }

        private string CurrentPath { get; }
        public Application App { get; }
        public UIA3Automation Automation { get; }
        public Window MainWindow { get; }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(CurrentPath, App, Automation);
        }

        internal void Deconstruct(out string currentPath, out Application app, out UIA3Automation automation,
            out Window mainWindow)
        {
            currentPath = CurrentPath;
            app = App;
            automation = Automation;
            mainWindow = MainWindow;
        }
    }
}