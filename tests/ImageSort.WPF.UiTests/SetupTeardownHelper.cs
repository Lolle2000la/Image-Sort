using FlaUI.Core;
using FlaUI.UIA3;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;

namespace ImageSort.WPF.UiTests
{
    static class SetupTeardownHelper
    {
        public static (string, Application, UIA3Automation) Setup()
        {
            var currentPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Temporary State", Guid.NewGuid().ToString());

            CopyFolder(Path.GetFullPath("MockState"), currentPath);

            var app = Application.Launch(new ProcessStartInfo("Image Sort.exe", $"\"{currentPath}\""));
            var automation = new UIA3Automation();

            return (currentPath, app, automation);
        }

        public static void TearDown(string currentPath, Application app, UIA3Automation automation)
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
