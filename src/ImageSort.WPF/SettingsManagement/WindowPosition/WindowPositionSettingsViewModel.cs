using System.Windows;
using ImageSort.SettingsManagement;
using ReactiveUI;

namespace ImageSort.WPF.SettingsManagement.WindowPosition
{
    public class WindowPositionSettingsViewModel<TWindow> : SettingsGroupViewModelBase
        where TWindow : Window
    {
        // size
        private int _height = 600;

        private bool _isMaximized;

        // position
        private int _left = 100;

        // used to ensure that when the window count changes the window will still be visible (e.g. when the display count changes everything will be reset)
        private int _screenCount;

        private int _top = 100;

        private int _width = 1000;
        public override string Name => typeof(TWindow).Name;

        public override string Header => typeof(TWindow).Name;

        public override bool IsVisible => false;

        public bool IsMaximized
        {
            get => _isMaximized;
            set => this.RaiseAndSetIfChanged(ref _isMaximized, value);
        }

        public int Left
        {
            get => _left;
            set => this.RaiseAndSetIfChanged(ref _left, value);
        }

        public int Top
        {
            get => _top;
            set => this.RaiseAndSetIfChanged(ref _top, value);
        }

        public int Height
        {
            get => _height;
            set => this.RaiseAndSetIfChanged(ref _height, value);
        }

        public int Width
        {
            get => _width;
            set => this.RaiseAndSetIfChanged(ref _width, value);
        }

        public int ScreenCount
        {
            get => _screenCount;
            set => this.RaiseAndSetIfChanged(ref _screenCount, value);
        }
    }
}