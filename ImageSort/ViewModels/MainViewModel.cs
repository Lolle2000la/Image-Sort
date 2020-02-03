using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Reactive;
using System.Text;

namespace ImageSort.ViewModels
{
    public class MainViewModel : ReactiveObject
    {
        private FoldersViewModel _foldersViewModel;
        public FoldersViewModel Folders
        {
            get => _foldersViewModel;
            set => this.RaiseAndSetIfChanged(ref _foldersViewModel, value);
        }

        public ReactiveCommand<Unit, Unit> OpenCurrentlySelectedFolder { get; }

        private readonly ObservableAsPropertyHelper<ImagesViewModel> _images;
        public ImagesViewModel Images;

        public MainViewModel()
        {
            OpenCurrentlySelectedFolder = ReactiveCommand.Create(() =>
            {
                Folders.CurrentFolder = Folders.Selected;
            });
        }
    }
}
