using System;
using System.Diagnostics;
using System.IO;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Tools;
using FlaUI.UIA3;

namespace ImageSort.WPF.UiTests
{
    internal static class SetupTeardownHelper
    {
        public static (string, Application, UIA3Automation, Window) Setup()
        {
            var currentPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Temporary State",
                Guid.NewGuid().ToString());

            CopyFolder(Path.GetFullPath("MockState"), currentPath);

            // ensure previous config file is removed
            if (File.Exists(@".\ui_test_config.json")) File.Delete(@".\ui_test_config.json");

            var procInfo = new ProcessStartInfo("Image Sort.exe", $"\"{currentPath}\"");

            // ensures the app is not affected by and does not affect global config file
            procInfo.EnvironmentVariables.Add("UI_TEST", "true");

            var app = Application.Launch(procInfo);
            var automation = new UIA3Automation();

            app.WaitWhileBusy();

            var mainWindow = Retry.WhileNull(() =>
            {
                var allWindows = app.GetAllTopLevelWindows(automation);

                if (allWindows.Length > 0) return allWindows[0];

                return null;
            }, TimeSpan.FromSeconds(30), null, true).Result;

            app.WaitWhileBusy();
            app.WaitWhileMainHandleIsMissing();
            mainWindow.WaitUntilClickable();

            mainWindow.Focus();

            while (currentPath == null || app == null || automation == null || mainWindow == null)
            {
            }

            ControlHelper.App = app;
            ControlHelper.MainWindow = mainWindow;

            return (currentPath, app, automation, mainWindow);
        }

        public static void TearDown(string currentPath, Application app, UIA3Automation automation)
        {
            app.Close();
            automation.Dispose();
            app.Dispose();

            Directory.Delete(currentPath, true);
        }

        private static void CopyFolder(string sourceFolder, string destFolder)
        {
            if (!Directory.Exists(destFolder)) Directory.CreateDirectory(destFolder);

            var files = Directory.GetFiles(sourceFolder);

            foreach (var file in files)
            {
                var name = Path.GetFileName(file);
                var dest = Path.Combine(destFolder, name);
                File.Copy(file, dest);
            }

            var folders = Directory.GetDirectories(sourceFolder);

            foreach (var folder in folders)
            {
                var name = Path.GetFileName(folder);
                var dest = Path.Combine(destFolder, name);
                CopyFolder(folder, dest);
            }
        }
    }
}