using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;
using FlaUI.Core;
using FlaUI.UIA3;
using Xunit;

namespace ImageSort.WPF.UiTests
{
    public class SearchTests : IDisposable
    {
        private readonly Application app;
        private readonly UIA3Automation automation;
        private readonly string currentPath;

        public SearchTests()
        {
            (currentPath, app, automation) = SetupTeardownHelper.Setup();
        }

        [Fact(DisplayName = "Filters out images correctly")]
        public void FiltersOutImagesCorrectly()
        {
        }

        public void Dispose()
        {
            SetupTeardownHelper.TearDown(currentPath, app, automation);
        }
    }
}
