using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.WPF.SettingsManagement.WindowPosition
{
    public class WindowPositionSettingsViewModel : SettingsGroupViewModelBase
    {
        public override string Name => "WindowPosition";

        public override string Header => "Window position";

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
        private int _width = 100;
        public int Width
        {
            get => _width;
            set => this.RaiseAndSetIfChanged(ref _width, value);
        }

        private int _height = 100;
        public int Height
        {
            get => _height;
            set => this.RaiseAndSetIfChanged(ref _height, value);
        }
    }
}
