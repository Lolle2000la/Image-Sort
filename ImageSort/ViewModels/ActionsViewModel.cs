using ImageSort.Actions;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Reactive;
using System.Text;

namespace ImageSort.ViewModels
{
    public class ActionsViewModel : ReactiveObject
    {
        private readonly Stack<IReversibleAction> done;
        private readonly Stack<IReversibleAction> undone;

        private readonly ObservableAsPropertyHelper<string> lastDone;
        public string LastDone => lastDone.Value;

        private readonly ObservableAsPropertyHelper<string> lastUndone;
        public string LastUndone => lastUndone.Value;

        public ReactiveCommand<IReversibleAction, Unit> Execute { get; }
        public ReactiveCommand<Unit, Unit> Undo { get; }
        public ReactiveCommand<Unit, Unit> Redo { get; }

        public ActionsViewModel()
        {
            throw new NotImplementedException();
        }
    }
}
