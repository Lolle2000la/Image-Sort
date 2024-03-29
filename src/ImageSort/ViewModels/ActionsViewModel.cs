﻿using ImageSort.Actions;
using ImageSort.Localization;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Reactive;
using System.Reactive.Linq;

namespace ImageSort.ViewModels;

public class ActionsViewModel : ReactiveObject
{
    private readonly Stack<IReversibleAction> done = new Stack<IReversibleAction>();
    private readonly Stack<IReversibleAction> undone = new Stack<IReversibleAction>();

    private readonly ObservableAsPropertyHelper<string> lastDone;
    public string LastDone => lastDone.Value;

    private readonly ObservableAsPropertyHelper<string> lastUndone;
    public string LastUndone => lastUndone.Value;

    public Interaction<string, Unit> NotifyUserOfError { get; } = new Interaction<string, Unit>();

    public ReactiveCommand<IReversibleAction, Unit> Execute { get; }
    public ReactiveCommand<Unit, Unit> Undo { get; }
    public ReactiveCommand<Unit, Unit> Redo { get; }
    public ReactiveCommand<Unit, Unit> Clear { get; }

    public ActionsViewModel()
    {
        Execute = ReactiveCommand.CreateFromTask<IReversibleAction>(async action =>
        {
            try
            {
                action.Act();
            }
            catch (Exception ex)
            {
                await NotifyUserOfError.Handle(Text.CouldNotActErrorText
                    .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                    .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));

                return;
            }

            done.Push(action);

            undone.Clear();
        });

        Undo = ReactiveCommand.CreateFromTask(async () =>
        {
            if (done.TryPop(out var action))
            {
                try
                {
                    action.Revert();
                }
                catch (Exception ex)
                {
                    await NotifyUserOfError.Handle(Text.CouldNotUndoErrorText
                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                        .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));

                    return;
                }

                undone.Push(action);
            }
        });

        Redo = ReactiveCommand.CreateFromTask(async () =>
        {
            if (undone.TryPop(out var action))
            {
                try
                {
                    action.Act();
                }
                catch (Exception ex)
                {
                    await NotifyUserOfError.Handle(Text.CouldNotRedoErrorText
                        .Replace("{ErrorMessage}", ex.Message, StringComparison.OrdinalIgnoreCase)
                        .Replace("{ActMessage}", action.DisplayName, StringComparison.OrdinalIgnoreCase));

                    return;
                }

                done.Push(action);
            }
        });

        Clear = ReactiveCommand.Create(() =>
        {
            done.Clear();
            undone.Clear();
        });

        var historyChanges = Execute.Merge(Undo).Merge(Redo).Merge(Clear);

        lastDone = historyChanges
            .Select(_ =>
            {
                if (done.TryPeek(out var action)) return action.DisplayName;

                return null;
            })
            .ToProperty(this, vm => vm.LastDone);

        lastUndone = historyChanges
            .Select(_ =>
            {
                if (undone.TryPeek(out var action)) return action.DisplayName;

                return null;
            })
            .ToProperty(this, vm => vm.LastUndone);
    }
}