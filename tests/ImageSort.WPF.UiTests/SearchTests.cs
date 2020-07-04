using System.Collections.Generic;
using System.IO;
using System.Linq;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.UIA3;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    [Collection("App collection")]
    public class SearchTests
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;
        private readonly Window mainWindow;

        public SearchTests(AppFixture appFixture)
        {
            (currentPath, app, automation, mainWindow) = appFixture;
        }

        [Fact(DisplayName = "Filters out images correctly")]
        public void FiltersOutImagesCorrectly()
        {
            var search = mainWindow.FindFirstDescendant(cf => cf.ByAutomationId("SearchTerm")).AsTextBox();
            search.Focus();
            search.Text = ".jpg";

            var images = mainWindow.GetImages()
                .Select(n => Path.GetFileName(n));

            Assert.DoesNotContain("mock 4.png", images);
        }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(currentPath, app, automation);
        }
    }
}
