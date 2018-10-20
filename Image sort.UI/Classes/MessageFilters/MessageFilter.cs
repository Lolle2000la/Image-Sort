using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Interop;

namespace Image_sort.UI.Classes.MessageFilters
{
    class MessageFilter<TMessageParam>
    {
        private Action<TMessageParam> messageHandler = null;

        /// <summary>
        /// Constructs a new instance of the message filter.
        /// </summary>
        public MessageFilter()
        {
            ComponentDispatcher.ThreadFilterMessage += OnThreadFilterMessageRaised;
        }

        /// <summary>
        /// Constructs a new instance of the message filter.
        /// </summary>
        /// <param name="messageHandler">
        /// Handler called, when the specified message is raised.
        /// </param>
        public MessageFilter(Action<TMessageParam> messageHandler)
        {
            this.messageHandler = messageHandler;

            ComponentDispatcher.ThreadFilterMessage += OnThreadFilterMessageRaised;
        }

        /// <summary>
        /// Processes the messages, handles them and calls the <see cref="messageHandler"/>.
        /// </summary>
        /// <param name="msg"></param>
        /// <param name="handled"></param>
        protected virtual void OnThreadFilterMessageRaised(ref MSG msg, ref bool handled)
        {
            LogMessage(msg, handled);
        }

        [Conditional("DEBUG")]
        private void LogMessage(MSG msg, bool handled)
        {
            Debug.WriteLine($"Message {msg.message} raised with lParam={msg.lParam} and wParam={msg.wParam}; Handled={handled}");
        }
        
        /// <summary>
        /// Raises the action with the message param, when given.
        /// </summary>
        /// <param name="messageParam">The parameter, that the action should be raised with.</param>
        protected void RaiseActionWith(TMessageParam messageParam)
        {
            messageHandler?.Invoke(messageParam);
        }
    }
}
