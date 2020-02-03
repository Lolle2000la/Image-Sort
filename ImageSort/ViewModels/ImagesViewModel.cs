using ReactiveUI;

namespace ImageSort.ViewModels
{
    public class ImagesViewModel : ReactiveObject
    {
        private string _currentPath;
        public string CurrentFolder
        {
            get => _currentPath;
            set => this.RaiseAndSetIfChanged(ref _currentPath, value);
        }
    }
}