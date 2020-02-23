using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media.Imaging;

namespace Image_sort.UI.Classes
{
    static class ShellFileLoader
    {
        public const int MaxPath = 259;
        public const uint FILE_ATTRIBUTE_DIRECTORY = 0x00000010;


        [DllImport("shell32.dll", CharSet = CharSet.Auto)]
        public static extern UIntPtr SHGetFileInfo(string pszPath, uint dwFileAttributes, ref SHFILEINFO psfi, uint cbfileInfo, ShellGetFileInfoFlags uFlags);

        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static extern bool DestroyIcon(IntPtr hIcon);

        /// <summary>
        /// Contains information about a file object.
        /// </summary>
        /// <seealso cref="http://msdn.microsoft.com/en-us/library/bb759792%28v=VS.85%29.aspx"/>
        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
        public struct SHFILEINFO
        {
            /// <summary>
            /// The size of the SHFILEINFO structure.
            /// </summary>
            public static readonly uint Size = (uint) Marshal.SizeOf(typeof(SHFILEINFO));

            /// <summary>
            /// A handle to the icon that represents the file. You are responsible for destroying this handle with DestroyIcon when you no longer need it.
            /// </summary>
            public IntPtr hIcon;

            /// <summary>
            /// The index of the icon image within the system image list.
            /// </summary>
            private int iIcon;

            /// <summary>
            /// An array of values that indicates the attributes of the file object.
            /// </summary>
            private uint dwAttributes;

            /// <summary>
            /// A string that contains the name of the file as it appears in the Windows Shell, or the path and file name of the file that contains the icon representing the file.
            /// </summary>
            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = MaxPath)]
            private string szDisplayName;

            /// <summary>
            /// A string that describes the type of file.
            /// </summary>
            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 80)]
            private string szTypeName;
        };


        [Flags]
        public enum ShellGetFileInfoFlags : uint
        {
            /// <summary>
            /// Modify SHGFI_ICON, causing the function to retrieve the file's large icon. The SHGFI_ICON flag must also be set.
            /// </summary>
            LargeIcon = 0x0,
            /// <summary>
            /// Modify SHGFI_ICON, causing the function to retrieve the file's small icon. Also used to modify SHGFI_SYSICONINDEX, 
            /// causing the function to return the handle to the system image list that contains small icon images. The SHGFI_ICON 
            /// and/or SHGFI_SYSICONINDEX flag must also be set.
            /// </summary>
            SmallIcon = 0x1,
            /// <summary>
            /// Retrieve the handle to the icon that represents the file and the index of the icon within the system image list. The handle is copied to 
            /// the hIcon member of the structure specified by psfi, and the index is copied to the iIcon member.
            /// </summary>
            Icon = 0x100,
            /// <summary>
            /// Indicates that the function should not attempt to access the file specified by pszPath. Rather, it 
            /// should act as if the file specified by pszPath exists with the file attributes passed in dwFileAttributes. 
            /// This flag cannot be combined with the SHGFI_ATTRIBUTES, SHGFI_EXETYPE, or SHGFI_PIDL flags.
            /// </summary>
            UseFileAttributes = 0x10
        }

        public static Bitmap GetThumbnailFromShell (string path)
        {
            SHFILEINFO info = new SHFILEINFO();
            // get the file info from the windows apu
            SHGetFileInfo(path, FILE_ATTRIBUTE_DIRECTORY,
                ref info, (uint) Marshal.SizeOf(info), ShellGetFileInfoFlags.Icon | ShellGetFileInfoFlags.SmallIcon);

            // save it into a bitmap
            Bitmap thumbnail = ((Icon) Icon.FromHandle(info.hIcon).Clone()).ToBitmap();

            // destroy the icon, as it isn't needed anymore
            DestroyIcon(info.hIcon);

            return thumbnail;
        }

        public static BitmapImage GetThumbnailFromShellForWpf (string path)
        {
            var thumbnail = GetThumbnailFromShell(path);

            using var memory = new MemoryStream();

            thumbnail.Save(memory, ImageFormat.Png);
            memory.Position = 0;

            var bitmapImage = new BitmapImage();
            bitmapImage.BeginInit();
            bitmapImage.StreamSource = memory;
            bitmapImage.CacheOption = BitmapCacheOption.OnLoad;
            bitmapImage.EndInit();

            return bitmapImage;
        }
    }
}
