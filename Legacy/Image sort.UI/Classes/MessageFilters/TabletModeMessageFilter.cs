using System;
using System.Threading.Tasks;
using Microsoft.Win32;

namespace Image_sort.UI.Classes.MessageFilters
{
    internal class TabletModeMessageFilter : MessageFilter<bool>
    {
        public TabletModeMessageFilter() : base() { }

        public TabletModeMessageFilter(Action<bool> messageHandler) : base(messageHandler)
        {
            SystemEvents.UserPreferenceChanged += OnUserPreferenceChanged;
        }

        ~TabletModeMessageFilter()
        {
            // necessary to prevent memory leaks
            SystemEvents.UserPreferenceChanged -= OnUserPreferenceChanged;
        }

        /// <summary>
        /// Gets whether Tablet Mode is enabled. 
        /// Always returns false if tablet mode isn't supported.
        /// </summary>
        public static bool TabletMode
        {
            get
            {
                try
                {
                    RegistryKey key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\ImmersiveShell");
                    return ((int) key.GetValue("TabletMode")) == 1;
                }
                // there is no tablet mode support, or it isn't available.
                catch
                {
                    return false;
                }
            }
        }

        /// <summary>
        /// Gets whether Tablet Mode is enabled. 
        /// Always returns false if tablet mode isn't supported.
        /// </summary>
        /// <param name="waitMilliseconds">Waits that amount of time before getting <see cref="TabletMode"/>.</param>
        /// <returns>whether tablet mode is enabled or not</returns>
        private static async Task<bool> GetTabletModeAsync(int waitMilliseconds = 500)
        {
            await Task.Delay(waitMilliseconds);
            return TabletMode;
        }

        // keeps track of the last state, that the tablet mode had.
        // ensures that there aren't unnecessary calls to messageHandler.
        private bool lastTabletModeState = TabletMode;

        private async void OnUserPreferenceChanged(object sender, UserPreferenceChangedEventArgs e)
        {
            bool tabletModeEnabled = await GetTabletModeAsync();
            // prevent unnecessary tablet mode calls.
            if (lastTabletModeState != tabletModeEnabled)
            {
                lastTabletModeState = tabletModeEnabled;
                RaiseActionWith(tabletModeEnabled);
            }
        }
    }
}
