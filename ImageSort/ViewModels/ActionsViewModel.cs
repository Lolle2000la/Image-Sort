using ImageSort.Actions;
using ReactiveUI;
using System.Collections.Generic;
using System.Reactive;
using System.Reactive.Linq;

namespace ImageSort.ViewModels
{
    public class ActionsViewModel : ReactiveObject
    {
        private readonly Stack<IReversibleAction> done = new Stack<IReversibleAction>();
        private readonly Stack<IReversibleAction> undone = new Stack<IReversibleAction>();

        private readonly ObservableAsPropertyHelper<string> lastDone;
        public string LastDone => lastDone.Value;

        private readonly ObservableAsPropertyHelper<string> lastUndone;
        public string LastUndone => lastUndone.Value;

        public ReactiveCommand<IReversibleAction, Unit> Execute { get; }
        public ReactiveCommand<Unit, Unit> Undo { get; }
        public ReactiveCommand<Unit, Unit> Redo { get; }

        public ActionsViewModel()
        {
            Execute = ReactiveCommand.Create<IReversibleAction>(action =>
            {
                action.Act();

                done.Push(action);

                undone.Clear();
            });

            Undo = ReactiveCommand.Create(() =>
            {
                if (done.TryPop(out var action))
                {
                    action.Revert();

                    undone.Push(action);
                }
            });

            Redo = ReactiveCommand.Create(() =>
            {
                if (undone.TryPop(out var action))
                {
                    action.Act();

                    done.Push(action);
                }
            });

            var historyChanges = Execute.Merge(Undo).Merge(Redo);

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
}
