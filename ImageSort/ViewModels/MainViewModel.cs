﻿using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Reactive;
using System.Reactive.Linq;
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

        private ImagesViewModel _images;
        public ImagesViewModel Images 
        {
            get => _images;
            set => this.RaiseAndSetIfChanged(ref _images, value);
        }

        public MainViewModel()
        {
            this.WhenAnyValue(x => x.Folders.CurrentFolder)
                .Where(f => f != null)
                .Select(f => f.Path)
                .Subscribe(f => 
                {
                    Images.CurrentFolder = f;
                });

            var canOpenCurrentlySelectedFolder = this.WhenAnyValue(x => x.Folders)
                .Where(f => f != null)
                .SelectMany(f => f.WhenAnyValue(x => x.Selected, x => x.CurrentFolder, (s, c) => new { Selected = s, CurrentFolder = c }))
                .Where(f => f != null)
                .Select(f => f.Selected != null && f.Selected != f.CurrentFolder);

            OpenCurrentlySelectedFolder = ReactiveCommand.Create(() =>
            {
                Folders.CurrentFolder = Folders.Selected;
            }, canOpenCurrentlySelectedFolder);
        }
    }
}
