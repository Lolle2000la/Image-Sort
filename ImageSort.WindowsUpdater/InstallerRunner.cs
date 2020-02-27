using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;
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

            var fs = File.Create(Path.Combine(tempFolder, "imagesort.exe"));

            await installer.CopyToAsync(fs);

            fs.Close();

            RunSetup(setupPath);

            Environment.Exit(0);
        }

        private static void RunSetup(string path)
        {
            Process.Start("cmd", $"/c @ping -n 5 localhost> nul & msiexec /i \"{path}\" TARGETDIR=\"{AppDomain.CurrentDomain.BaseDirectory}\" /passive");
        }
    }
}
