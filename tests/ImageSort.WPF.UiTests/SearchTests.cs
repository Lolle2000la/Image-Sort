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
            currentPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Temporary State", Guid.NewGuid().ToString());

            CopyFolder(Path.GetFullPath("MockState"), currentPath);

            app = Application.Launch(new ProcessStartInfo("Image Sort.exe", $"\"{currentPath}\""));
            automation = new UIA3Automation();
        }

        [Fact(DisplayName = "Filters out images correctly")]
        public void FiltersOutImagesCorrectly()
        {
        }

        public void Dispose()
        {
            automation.Dispose();
            app.Dispose();

            Directory.Delete(currentPath, true);
        }

        private static void CopyFolder(string sourceFolder, string destFolder)
        {
            if (!Directory.Exists(destFolder)) Directory.CreateDirectory(destFolder);

            var files = Directory.GetFiles(sourceFolder);

            foreach (string file in files)
            {
                string name = Path.GetFileName(file);
                string dest = Path.Combine(destFolder, name);
                File.Copy(file, dest);
            }

            var folders = Directory.GetDirectories(sourceFolder);

            foreach (string folder in folders)
            {
                string name = Path.GetFileName(folder);
                string dest = Path.Combine(destFolder, name);
                CopyFolder(folder, dest);
            }
        }
    }
}
