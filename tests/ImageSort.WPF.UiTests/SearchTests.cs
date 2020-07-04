using System.IO;
using System.Linq;
using FlaUI.Core.AutomationElements;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    [Collection("App collection")]
    public class SearchTests
    {
        private readonly Window mainWindow;

        public SearchTests(AppFixture appFixture)
        {
            (_, _, _, mainWindow) = appFixture;
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
    }
}
