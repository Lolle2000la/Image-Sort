using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;

namespace ImageSort.WindowsUpdater
{
    public static class InstallerRunner
    {
        public static async Task RunAsync(Stream installer)
        {
            if (installer == null) throw new ArgumentNullException(nameof(installer));

            var tempFolder = Path.Combine(Path.GetTempPath(), "Image Sort");
            var setupPath = Path.Combine(tempFolder, $"ImageSort.{(Environment.Is64BitProcess ? "x64" : "x86")}.msi");

            if (!Directory.Exists(tempFolder)) Directory.CreateDirectory(tempFolder);

            var fs = File.Create(Path.Combine(tempFolder, setupPath));

            await installer.CopyToAsync(fs);

            await fs.DisposeAsync();

            RunSetup(setupPath);

            Environment.Exit(0);
        }

        private static void RunSetup(string path)
        {
            var processStartInfo = new ProcessStartInfo("msiexec", $"/i \"{path}\" TARGETDIR=\"{AppDomain.CurrentDomain.BaseDirectory}\" /passive")
            {
                Verb = "runas",
                UseShellExecute = true
            };

            Process.Start(processStartInfo);
        }

        public static void CleanUpInstaller()
        {
            var setupPath = Path.Combine(Path.GetTempPath(), "Image Sort", $"ImageSort.{(Environment.Is64BitProcess ? "x64" : "x86")}.msi");

            if (File.Exists(setupPath)) File.Delete(setupPath);
        }
    }
}