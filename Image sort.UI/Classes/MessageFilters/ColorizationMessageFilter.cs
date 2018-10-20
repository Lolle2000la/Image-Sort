using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Interop;
using System.Windows.Media;

namespace Image_sort.UI.Classes.MessageFilters
{
    class ColorizationMessageFilter : MessageFilter<Color>
    {

        /// <summary>
        /// Message-code for changing colorization.
        /// </summary>
        private const uint WM_DWMCOLORIZATIONCOLORCHANGED = 0x0320;

        protected override void OnThreadFilterMessageRaised(ref MSG msg, ref bool handled)
        {
            if (msg.message == WM_DWMCOLORIZATIONCOLORCHANGED)
            {
                int wParam = msg.wParam.ToInt32();
                RaiseActionWith(Color.FromArgb(255,
                    (byte) (wParam >> 16),
                    (byte) (wParam >> 8),
                    (byte) wParam));
                handled = true;
            }

            handled = false;
            
            base.OnThreadFilterMessageRaised(ref msg, ref handled);
        }
    }
}
