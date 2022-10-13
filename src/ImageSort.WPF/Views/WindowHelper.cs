using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Forms;
using ImageSort.SettingsManagement;
using ImageSort.WPF.SettingsManagement.WindowPosition;
using Splat;

namespace ImageSort.WPF.Views;

internal static class WindowHelper
{
    private static WindowPositionSettingsViewModel<TWindow> GetWindowPostion<TWindow>() where TWindow : Window
    {
        return Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
            .OfType<WindowPositionSettingsViewModel<TWindow>>()
            .FirstOrDefault();
    }

    public static void RestoreWindowState<TWindow>(this TWindow window) where TWindow : Window
    {
        var windowPosition = GetWindowPostion<TWindow>();

        if (windowPosition == null) return;

        var screenCount = Screen.AllScreens.Length;

        // ensure when the number of screen was changed the window will still be visible
        if (windowPosition.ScreenCount != screenCount)
        {
            windowPosition.ScreenCount = screenCount;
        }
        else
        {
            window.Left = windowPosition.Left;
            window.Top = windowPosition.Top;
        }

        window.WindowState = windowPosition.IsMaximized ? WindowState.Maximized : WindowState.Normal;
        window.Height = windowPosition.Height;
        window.Width = windowPosition.Width;
    }

    public static void SaveWindowState<TWindow>(this TWindow window) where TWindow : Window
    {
        var windowPosition = GetWindowPostion<TWindow>();

        if (windowPosition == null) return;

        windowPosition.IsMaximized = window.WindowState == WindowState.Maximized;

        if (window.WindowState == WindowState.Maximized) window.WindowState = WindowState.Normal;

        windowPosition.Left = (int) window.Left;
        windowPosition.Top = (int) window.Top;
        windowPosition.Height = (int) window.Height;
        windowPosition.Width = (int) window.Width;

        // record the screen count at the time.
        windowPosition.ScreenCount = Screen.AllScreens.Length;
    }
}