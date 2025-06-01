using ImageSort.Actions;
using ImageSort.FileSystem; // Added for IFileSystem
using ImageSort.Localization;
using ReactiveUI;
using Splat; // Added for Locator
using System;
using System.Collections.Generic;
using System.Reactive;
using System.Reactive.Linq;
using System.Reactive.Subjects; // Added for Subject

namespace ImageSort.ViewModels;

public class ActionsViewModel : ReactiveObject
{
    private readonly Stack<IReversibleAction> done = new Stack<IReversibleAction>();
    private readonly Stack<IReversibleAction> undone = new Stack<IReversibleAction>();

    private readonly ImagesViewModel imagesViewModel; // Added
    private readonly FoldersViewModel foldersViewModel; // Added
    private readonly IFileSystem fileSystem; // Added

    private readonly ObservableAsPropertyHelper<string> lastDone;
    public string LastDone => lastDone.Value;

    private readonly ObservableAsPropertyHelper<string> lastUndone;
    public string LastUndone => lastUndone.Value;

    private readonly ObservableAsPropertyHelper<bool> _canUndo;
    public bool CanUndo => _canUndo.Value;

    private readonly ObservableAsPropertyHelper<bool> _canRedo;
    public bool CanRedo => _canRedo.Value;

    public Interaction<string, Unit> NotifyUserOfError { get; } = new Interaction<string, Unit>();

    public ReactiveCommand<IReversibleAction, Unit> Execute { get; }
    public ReactiveCommand<Unit, Unit> Undo { get; }
    public ReactiveCommand<Unit, Unit> Redo { get; }
    public ReactiveCommand<Unit, Unit> Clear { get; }

    // New commands for moving images
    public ReactiveCommand<Unit, Unit> Move { get; private set; } // For moving to current selected folder (grid)
    public ReactiveCommand<string, Unit> MoveImageToFolder { get; private set; } // For moving to a specific folder path (e.g., pinned folders)

    private readonly Subject<Unit> _historyChangedSignal = new Subject<Unit>();

    public ActionsViewModel(ImagesViewModel imagesViewModel = null, 
                            FoldersViewModel foldersViewModel = null, 
                            IFileSystem fileSystem = null)
    {
        this.imagesViewModel = imagesViewModel ?? Locator.Current.GetService<ImagesViewModel>();
        this.foldersViewModel = foldersViewModel ?? Locator.Current.GetService<FoldersViewModel>();
        this.fileSystem = fileSystem ?? Locator.Current.GetService<IFileSystem>();

        var canUndoObservable = _historyChangedSignal
            .Select(_ => done.Count > 0)
            .StartWith(done.Count > 0);

        var canRedoObservable = _historyChangedSignal
            .Select(_ => undone.Count > 0)
            .StartWith(undone.Count > 0);

        Execute = ReactiveCommand.CreateFromTask<IReversibleAction>(async action =>
        {
            try
            {
                action.Act();
                done.Push(action);
                undone.Clear(); // Clear redo stack on new action
                _historyChangedSignal.OnNext(Unit.Default);
            }
            catch (Exception ex)
            {
                await NotifyUserOfError.Handle(Text.CouldNotActErrorText
                    .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                    .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));
            }
        });

        Undo = ReactiveCommand.CreateFromTask(async () =>
        {
            if (done.TryPop(out var action))
            {
                try
                {
                    action.Revert();
                    undone.Push(action);
                    _historyChangedSignal.OnNext(Unit.Default);
                }
                catch (Exception ex)
                {
                    await NotifyUserOfError.Handle(Text.CouldNotUndoErrorText
                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                        .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));
                    // Signal change even if revert fails, because 'done' stack changed.
                    _historyChangedSignal.OnNext(Unit.Default);
                }
            }
        }, canUndoObservable);

        Redo = ReactiveCommand.CreateFromTask(async () =>
        {
            if (undone.TryPop(out var action))
            {
                try
                {
                    action.Act(); // Re-apply the action
                    done.Push(action);
                    _historyChangedSignal.OnNext(Unit.Default);
                }
                catch (Exception ex)
                {
                    await NotifyUserOfError.Handle(Text.CouldNotRedoErrorText
                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                        .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));
                    // Signal change even if re-act fails, because 'undone' stack changed.
                    _historyChangedSignal.OnNext(Unit.Default);
                }
            }
        }, canRedoObservable);

        Clear = ReactiveCommand.Create(() =>
        {
            done.Clear();
            undone.Clear();
            _historyChangedSignal.OnNext(Unit.Default);
        });

        InitializeMoveCommands();

        _canUndo = canUndoObservable.ToProperty(this, vm => vm.CanUndo);
        _canRedo = canRedoObservable.ToProperty(this, vm => vm.CanRedo);

        lastDone = _historyChangedSignal
            .Select(_ => done.TryPeek(out var action) ? action.DisplayName : null)
            .StartWith(done.TryPeek(out var action) ? action.DisplayName : null)
            .ToProperty(this, vm => vm.LastDone);

        lastUndone = _historyChangedSignal
            .Select(_ => undone.TryPeek(out var action) ? action.DisplayName : null)
            .StartWith(undone.TryPeek(out var undoneAction) ? undoneAction.DisplayName : null)
            .ToProperty(this, vm => vm.LastUndone);
    }

    private void InitializeMoveCommands()
    {
        // Observable that is true when both imagesViewModel and foldersViewModel are non-null.
        var viewModelsAvailable = this.WhenAnyValue(
            x => x.imagesViewModel,
            x => x.foldersViewModel,
            (imgVm, folVm) => imgVm != null && folVm != null)
            .DistinctUntilChanged();

        // canMove: True if view models are available, an image is selected, and a current folder path is set.
        var canMove = viewModelsAvailable
            .Select(areAvailable =>
            {
                if (!areAvailable) return Observable.Return(false);

                // Defer creation of the inner observable until subscription, 
                // and after we know imagesViewModel and foldersViewModel are not null.
                return Observable.Defer(() =>
                {
                    // Re-check for safety, though 'areAvailable' should mean they are not null.
                    if (this.imagesViewModel == null || this.foldersViewModel == null) return Observable.Return(false);

                    var selectedImageObservable = this.imagesViewModel.WhenAnyValue(vm => vm.SelectedImage)
                        .Select(img => !string.IsNullOrEmpty(img));

                    var currentFolderPathObservable = this.foldersViewModel.WhenAnyValue(vm => vm.CurrentFolder)
                        .Select(currentFolder => currentFolder != null && !string.IsNullOrEmpty(currentFolder.Path))
                        .DistinctUntilChanged();

                    return Observable.CombineLatest(
                        selectedImageObservable,
                        currentFolderPathObservable,
                        (imageSelected, pathValid) => imageSelected && pathValid
                    );
                });
            })
            .Switch() // Switch to the observable emitted by Select
            .StartWith(false)
            .DistinctUntilChanged();

        Move = ReactiveCommand.CreateFromTask(async () =>
        {
            if (this.imagesViewModel?.SelectedImage == null || this.foldersViewModel?.CurrentFolder?.Path == null || this.fileSystem == null) return;

            var moveAction = new MoveAction(
                this.imagesViewModel.SelectedImage,
                this.foldersViewModel.CurrentFolder.Path,
                this.fileSystem,
                this.imagesViewModel.OnImageMoved,
                (newPath, oldPath) => this.imagesViewModel.OnImageMoved(newPath, oldPath)
            );
            await Execute.Execute(moveAction);
        }, canMove);

        // Observable that is true when imagesViewModel is non-null.
        var imageViewModelAvailable = this.WhenAnyValue(x => x.imagesViewModel)
            .Select(imgVm => imgVm != null)
            .DistinctUntilChanged();

        // canMoveToFolder: True if imagesViewModel is available and an image is selected.
        var canMoveToFolder = imageViewModelAvailable
            .Select(imgVmAvailable =>
            {
                if (!imgVmAvailable) return Observable.Return(false);
                
                return Observable.Defer(() =>
                {
                    if (this.imagesViewModel == null) return Observable.Return(false);

                    return this.imagesViewModel.WhenAnyValue(vm => vm.SelectedImage)
                        .Select(img => !string.IsNullOrEmpty(img));
                });
            })
            .Switch()
            .StartWith(false)
            .DistinctUntilChanged();

        MoveImageToFolder = ReactiveCommand.CreateFromTask<string>(async (folderPath) =>
        {
            if (this.imagesViewModel?.SelectedImage == null || string.IsNullOrEmpty(folderPath) || this.fileSystem == null) return;

            var moveAction = new MoveAction(
                this.imagesViewModel.SelectedImage,
                folderPath,
                this.fileSystem,
                this.imagesViewModel.OnImageMoved,
                (newPath, oldPath) => this.imagesViewModel.OnImageMoved(newPath, oldPath)
            );
            await Execute.Execute(moveAction);
        }, canMoveToFolder);
    }
}