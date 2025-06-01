using ImageSort.Actions;
using ImageSort.FileSystem;
using ReactiveUI;
using Splat;
using System;
using System.Linq;
using System.Reactive;
using System.Reactive.Concurrency;
using System.Reactive.Linq;

namespace ImageSort.ViewModels;

public class MainViewModel : ReactiveObject
{
    public ActionsViewModel Actions { get; }
    public FoldersViewModel Folders { get; }
    public ImagesViewModel Images { get; }

    public Interaction<Unit, string> PickFolder { get; } = new Interaction<Unit, string>();

    public ReactiveCommand<Unit, Unit> OpenFolder { get; }
    public ReactiveCommand<Unit, Unit> OpenCurrentlySelectedFolder { get; }

    public ReactiveCommand<Unit, Unit> MoveImageToFolder { get; }

    public ReactiveCommand<Unit, Unit> DeleteImage { get; }

    public MainViewModel(FoldersViewModel foldersViewModel, ImagesViewModel imagesViewModel, ActionsViewModel actionsViewModel,
                         IFileSystem fileSystem = null, IRecycleBin recycleBin = null, IScheduler backgroundScheduler = null)
    {
        Folders = foldersViewModel ?? throw new ArgumentNullException(nameof(foldersViewModel));
        Images = imagesViewModel ?? throw new ArgumentNullException(nameof(imagesViewModel));
        Actions = actionsViewModel ?? throw new ArgumentNullException(nameof(actionsViewModel));

        fileSystem ??= Locator.Current.GetService<IFileSystem>();
        recycleBin ??= Locator.Current.GetService<IRecycleBin>();
        backgroundScheduler ??= RxApp.TaskpoolScheduler;

        // This subscription ensures ImagesViewModel.CurrentFolder is updated when FoldersViewModel.CurrentFolder changes.
        this.WhenAnyValue(x => x.Folders.CurrentFolder)
            .Where(f => f != null)
            .Select(f => f.Path)
            .Subscribe(folderPath =>
            {
                if (Images != null) Images.CurrentFolder = folderPath;
            });

        var canOpenCurrentlySelectedFolder = this.WhenAnyValue(x => x.Folders.Selected, x => x.Folders.CurrentFolder, 
                (s, c) => s != null && c != null && s != c && s.Path != c.Path) // Added s.Path != c.Path for clarity
            .DistinctUntilChanged();

        OpenCurrentlySelectedFolder = ReactiveCommand.Create(() =>
        {
            Folders.CurrentFolder = Folders.Selected;
        }, canOpenCurrentlySelectedFolder);

        OpenFolder = ReactiveCommand.CreateFromTask(async () =>
        {
            try
            {
                // The constructor for FolderTreeItemViewModel now requires a Func<FileSystemWatcher> and a nullable FolderTreeItemViewModel parent.
                // Providing null for the watcher factory and parent as this is a new root item.
                Folders.CurrentFolder = new FolderTreeItemViewModel(fileSystem, () => null, backgroundScheduler, null) { Path = await PickFolder.Handle(Unit.Default) };
            }
            catch (UnhandledInteractionException<Unit, string>) { }
        });

        var canMoveImageToFolderExecute = this.WhenAnyValue(x => x.Folders.Selected, x => x.Folders.CurrentFolder, x => x.Images.SelectedImage,
                (folderSelected, currentFolder, imageSelected) => 
                    folderSelected != null && 
                    currentFolder != null && 
                    folderSelected != currentFolder && 
                    folderSelected.Path != currentFolder.Path && // ensure paths are different
                    !string.IsNullOrEmpty(imageSelected))
            .DistinctUntilChanged();

        MoveImageToFolder = ReactiveCommand.CreateFromTask(async () =>
        {
            var moveAction = new MoveAction(Images.SelectedImage, Folders.Selected.Path, fileSystem,
                (o, n) => Images.RemoveImage(o), (n, o) => Images.InsertImage(o));

            var oldIndex = Images.SelectedIndex;

            await Actions.Execute.Execute(moveAction);

            if (oldIndex < Images.Images.Count) Images.SelectedIndex = oldIndex;
            else if (Images.Images.Any()) Images.SelectedIndex = 0;
        }, canMoveImageToFolderExecute);

        var canDeleteImageExecute = this.WhenAnyValue(x => x.Images.SelectedImage)
            .Select(i => !string.IsNullOrEmpty(i))
            .DistinctUntilChanged();

        DeleteImage = ReactiveCommand.CreateFromTask(async () =>
        {
            var deleteAction = new DeleteAction(Images.SelectedImage, fileSystem, recycleBin,
                o => Images.RemoveImage(o), o => Images.InsertImage(o));

            var oldIndex = Images.SelectedIndex;

            await Actions.Execute.Execute(deleteAction);

            if (oldIndex < Images.Images.Count) Images.SelectedIndex = oldIndex;
            else if (Images.Images.Any()) Images.SelectedIndex = 0;
        }, canDeleteImageExecute);

        this.WhenAnyValue(x => x.Folders.CurrentFolder)
            .Select(_ => Unit.Default)
            .ObserveOn(RxApp.MainThreadScheduler) // Ensure clear happens on UI thread if it affects UI
            .Subscribe(async _ => await Actions.Clear.Execute());

        // When a rename action is created by ImagesViewModel, execute it through ActionsViewModel
        Images.RenameImage
            .Where(a => a != null)
            .ObserveOn(RxApp.MainThreadScheduler) // Ensure execute happens on UI thread
            .SelectMany(action => Actions.Execute.Execute(action))
            .Subscribe();
    }
}