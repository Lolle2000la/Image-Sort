using System;
using System.Collections.Generic;
using System.Text;
using FlaUI.Core;
using FlaUI.UIA3;

namespace ImageSort.WPF.UiTests
{
    public class SearchTests
    {
        private readonly Application app;
        private readonly UIA3Automation automation;

        public SearchTests()
        {
            app = Application.Launch("Image Sort.exe");
            automation = new UIA3Automation();
        }



        ~SearchTests()
        {
            automation.Dispose();
            app.Dispose();
        }
    }
}
