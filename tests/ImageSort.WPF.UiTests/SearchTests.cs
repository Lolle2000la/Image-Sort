using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    public class SearchTests : IDisposable
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public SearchTests()
        {
            (currentPath, app, automation, mainWindow) = SetupTeardownHelper.Setup();
        }

        [Fact(DisplayName = "Filters out images correctly")]
        public void FiltersOutImagesCorrectly()
        {
            var search = mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("SearchTerm")).AsTextBox();
            search.Focus();
            search.Text = ".jpg";

            var images = mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("Images")).AsListBox()
                .FindAllChildren()
                .Select(e => e.AsListBoxItem())
                .Select(e => e.Name)
                .Select(n => Path.GetFileName(n));

            Assert.DoesNotContain("mock 4.png", images);
        }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(currentPath, app, automation);
        }
    }
}
