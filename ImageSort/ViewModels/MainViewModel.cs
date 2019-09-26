using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.ViewModels
{
    public class MainViewModel : ReactiveObject
    {
        public string _currentDirectory;
        public string CurrentDirectory
        {
            get => _currentDirectory;
            set => this.RaiseAndSetIfChanged(ref _currentDirectory, value);
        }

        private readonly ObservableAsPropertyHelper<ImagesViewModel> _images;
        public ImagesViewModel Images;

        
    }
}
