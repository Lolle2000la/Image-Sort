using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;
using System.Windows;

namespace ImageSort.WPF.SettingsManagement.WindowPosition
{
    public class WindowPositionSettingsViewModel<TWindow> : SettingsGroupViewModelBase
        where TWindow : Window
    {
        public override string Name => typeof(TWindow).Name;

        public override string Header => typeof(TWindow).Name;

        public override bool IsVisible => false;

        private bool _isMaximized = false;
        public bool IsMaximized
        {
            get => _isMaximized;
            set => this.RaiseAndSetIfChanged(ref _isMaximized, value);
        }

        // position
        private int _left = 100;
        public int Left
        {
            get => _left;
            set => this.RaiseAndSetIfChanged(ref _left, value);
        }

        private int _top = 100;
        public int Top
        {
            get => _top;
            set => this.RaiseAndSetIfChanged(ref _top, value);
        }

        // size
        private int _height = 600;
        public int Height
        {
            get => _height;
            set => this.RaiseAndSetIfChanged(ref _height, value);
        }

        private int _width = 1000;
        public int Width
        {
            get => _width;
            set => this.RaiseAndSetIfChanged(ref _width, value);
        }
    }
}
