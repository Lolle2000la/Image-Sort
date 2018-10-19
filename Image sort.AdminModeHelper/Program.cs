using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Win32;

namespace Image_sort.AdminModeHelper
{
    class Program
    {
        static void Main(string[] args)
        {
            // in the end this should result in passing the bool in "SHOWINCONTEXTMENU={bool}"
            // to the ShowInExplorerContextMenu property.
            if (args.Length == 1)
            {
                string varToChange = args[0];
                if (varToChange.StartsWith("SHOWINCONTEXTMENU="))
                {
                    ShowInExplorerContextMenu = bool.Parse(varToChange.Split('=')[1]);
                }
            }
        }

        public static bool ShowInExplorerContextMenu
        {
            set
            {
                // The path to where the key for registering in the explorer context menu must sit.
                const string baseKeyPath = "Directory\\shell\\ImageSort";
                string pathToExe = Path.Combine(Path.GetDirectoryName(System.Reflection.Assembly.GetEntryAssembly().Location), "Image sort.UI.exe");
                var key = Registry.ClassesRoot.OpenSubKey(baseKeyPath);
                if (value)
                {
                    if (key != null)
                    {
                        try
                        {
                            // removes the item from the context menu
                            Registry.ClassesRoot.DeleteSubKeyTree(baseKeyPath);
                        }
                        catch (Exception) { }
                    }
                }
                else
                {
                    if (key == null)
                    {
                        try
                        {
                            // adds the item to the context menu
                            key = Registry.ClassesRoot.CreateSubKey(baseKeyPath);
                            key.SetValue("", "Sort with Image Sort");
                            key.SetValue("Icon", $"\"{pathToExe}\"");
                            key.CreateSubKey("command").SetValue("", $"\"{pathToExe}\" -f \"%L\"");
                        }
                        catch (Exception) { }
                    }
                }
            }
        }
    }
}
