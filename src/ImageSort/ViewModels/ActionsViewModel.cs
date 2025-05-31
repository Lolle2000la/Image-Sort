using ImageSort.Actions;
using ImageSort.Localization;
using ReactiveUI;
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

    private readonly Subject<Unit> _historyChangedSignal = new Subject<Unit>();

    public ActionsViewModel()
    {
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

        _canUndo = canUndoObservable.ToProperty(this, vm => vm.CanUndo);
        _canRedo = canRedoObservable.ToProperty(this, vm => vm.CanRedo);

        lastDone = _historyChangedSignal
            .Select(_ => done.TryPeek(out var action) ? action.DisplayName : null)
            .StartWith(done.TryPeek(out var action) ? action.DisplayName : null)
            .ToProperty(this, vm => vm.LastDone);

        lastUndone = _historyChangedSignal
            .Select(_ => undone.TryPeek(out var action) ? action.DisplayName : null)
            .StartWith(undone.TryPeek(out var undoneAction) ? undoneAction.DisplayName : null) // Renamed variable here
            .ToProperty(this, vm => vm.LastUndone);
    }
}