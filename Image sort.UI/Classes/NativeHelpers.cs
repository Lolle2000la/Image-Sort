using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Media;

namespace Image_sort.UI.Classes
{
    class NativeHelpers
    {
        public static class NativeMethods
        {
            [DllImport("dwmapi.dll", EntryPoint = "#127")]
            public static extern void DwmGetColorizationParameters(ref DWMCOLORIZATIONcolors colors);
        }

        public struct DWMCOLORIZATIONcolors
        {
            public uint ColorizationColor,
                ColorizationAfterglow,
                ColorizationColorBalance,
                ColorizationAfterglowBalance,
                ColorizationBlurBalance,
                ColorizationGlassReflectionIntensity,
                ColorizationOpaqueBlend;
        }

        /// <summary>
        /// Gets the systems user set accent color.
        /// </summary>
        /// <param name="opaque">Get's, whether also the opaque color is supposed to be included.</param>
        /// <returns>the native <see cref="Color"/></returns>
        public static Color GetWindowColorizationColor(bool opaque)
        {
            DWMCOLORIZATIONcolors colors = new DWMCOLORIZATIONcolors();
            NativeMethods.DwmGetColorizationParameters(ref colors);

            return Color.FromArgb((byte) (opaque ? 255 : colors.ColorizationColor >> 24),
                (byte) (colors.ColorizationColor >> 16),
                (byte) (colors.ColorizationColor >> 8),
                (byte) colors.ColorizationColor);
        }
    }
}
