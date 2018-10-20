using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Interop;
using System.Windows.Media;

namespace Image_sort.UI.Classes.MessageFilters
{
    /// <summary>
    /// Filters the win32 messages for messages regarding accent color changes.
    /// </summary>
    class ColorizationMessageFilter : MessageFilter<Color>
    {
        public ColorizationMessageFilter() : base()
        {

        }

        public ColorizationMessageFilter(Action<Color> messageHandler) : base(messageHandler)
        {

        }

        /// <summary>
        /// Message-code for changing colorization.
        /// </summary>
        private const uint WM_DWMCOLORIZATIONCOLORCHANGED = 0x0320;

        protected override void OnThreadFilterMessageRaised(ref MSG msg, ref bool handled)
        {
            if (msg.message == WM_DWMCOLORIZATIONCOLORCHANGED)
            {
                int wParam = Environment.Is64BitProcess ? (int)msg.wParam.ToInt64() : msg.wParam.ToInt32();
                RaiseActionWith(Color.FromArgb(255,
                    (byte) (unchecked(wParam >> 16)),
                    (byte) (unchecked(wParam >> 8)),
                    (byte) unchecked(wParam)));
                handled = true;
            }

            handled = false;
            
            base.OnThreadFilterMessageRaised(ref msg, ref handled);
        }
    }
}
