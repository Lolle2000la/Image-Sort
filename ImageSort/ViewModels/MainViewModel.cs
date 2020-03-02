using ImageSort.Actions;
using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Linq;
using System.Reactive;
using System.Reactive.Linq;

namespace ImageSort.ViewModels
{
    public class MainViewModel : ReactiveObject
    {
        private ActionsViewModel actions;
        public ActionsViewModel Actions
        {
            get => actions;
            set => this.RaiseAndSetIfChanged(ref actions, value);
        }

        private FoldersViewModel _foldersViewModel;
        public FoldersViewModel Folders
        {
            get => _foldersViewModel;
            set => this.RaiseAndSetIfChanged(ref _foldersViewModel, value);
        }

        private ImagesViewModel _images;
        public ImagesViewModel Images
        {
            get => _images;
            set => this.RaiseAndSetIfChanged(ref _images, value);
        }

        public Interaction<Unit, string> PickFolder { get; } = new Interaction<Unit, string>();

        public ReactiveCommand<Unit, Unit> OpenFolder { get; }
        public ReactiveCommand<Unit, Unit> OpenCurrentlySelectedFolder { get; }

        public ReactiveCommand<Unit, Unit> MoveImageToFolder { get; }

        public ReactiveCommand<Unit, Unit> DeleteImage { get; }

        public MainViewModel(IFileSystem fileSystem = null, IRecycleBin recycleBin = null)
        {
            fileSystem ??= Locator.Current.GetService<IFileSystem>();
            recycleBin ??= Locator.Current.GetService<IRecycleBin>();

            this.WhenAnyValue(x => x.Images)
                .Where(i => i != null)
                .Subscribe(i =>
                {
                    this.WhenAnyValue(x => x.Folders.CurrentFolder)
                        .Where(f => f != null)
                        .Select(f => f.Path)
                        .Subscribe(f =>
                        {
                            i.CurrentFolder = f;
                        });
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

            OpenFolder = ReactiveCommand.CreateFromTask(async () =>
            {
                try
                {
                    Folders.CurrentFolder = new FolderTreeItemViewModel(fileSystem) { Path = await PickFolder.Handle(Unit.Default) };
                }
                catch (UnhandledInteractionException<Unit, string>) { }
            });

            var canMoveImageToFolderExecute = this.WhenAnyValue(x => x.Folders, x => x.Images, (f, i) => new { Folders = f, Images = i })
                .Where(fi => fi.Folders != null && fi.Images != null)
                .SelectMany(_ => Folders.WhenAnyValue(x => x.Selected, x => x.CurrentFolder, (s, c) => s != null && c != null && s != c)
                    .CombineLatest(Images.WhenAnyValue(x => x.SelectedImage), (f, s) => f && s != null));

            MoveImageToFolder = ReactiveCommand.CreateFromTask(async () =>
            {
                var moveAction = new MoveAction(Images.SelectedImage, Folders.Selected.Path, fileSystem,
                    (o, n) => Images.RemoveImage(o), (n, o) => Images.InsertImage(o));

                var oldIndex = Images.SelectedIndex;

                await Actions.Execute.Execute(moveAction);

                if (oldIndex < Images.Images.Count) Images.SelectedIndex = oldIndex;
                else if (Images.Images.Any()) Images.SelectedIndex = 0;
            }, canMoveImageToFolderExecute);

            var canDeleteImageExecute = this.WhenAnyValue(x => x.Images)
                .Where(i => i != null)
                .SelectMany(i => i.WhenAnyValue(x => x.SelectedImage))
                .Select(i => !string.IsNullOrEmpty(i));

            DeleteImage = ReactiveCommand.CreateFromTask(async () =>
            {
                var deleteAction = new DeleteAction(Images.SelectedImage, fileSystem, recycleBin,
                    o => Images.RemoveImage(o), o => Images.InsertImage(o));

                var oldIndex = Images.SelectedIndex;

                await Actions.Execute.Execute(deleteAction);

                if (oldIndex < Images.Images.Count) Images.SelectedIndex = oldIndex;
                else if (Images.Images.Any()) Images.SelectedIndex = 0;
            }, canDeleteImageExecute);

            this.WhenAnyValue(x => x.Folders, x => x.Actions)
                .Where(models => models.Item1 != null && models.Item2 != null)
                .SelectMany(_ => Folders.WhenAnyValue(x => x.CurrentFolder))
                .Select(_ => Unit.Default)
                .Subscribe(async _ => await Actions.Clear.Execute());
        }
    }
}
