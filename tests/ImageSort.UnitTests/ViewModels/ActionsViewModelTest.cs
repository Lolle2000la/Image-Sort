using System;
using System.Diagnostics.CodeAnalysis;
using System.Reactive;
using System.Reactive.Linq;
using System.Threading.Tasks;
using ImageSort.Actions;
using ImageSort.ViewModels;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.ViewModels;

public class ActionsViewModelTest
{
    [Fact(DisplayName =
        "Executes an action, adds it to the history, allows it to be undone and allows it to be redone and makes the last un-/done visible, also checking if clearing works.")]
    public async Task WorksCorrectly()
    {
        const string actionDisplayName = "Test action display name";

        var actionsVM = new ActionsViewModel();

        var actionMock = new Mock<IReversibleAction>();

        actionMock.Setup(a => a.Act()).Verifiable();
        actionMock.Setup(a => a.Revert()).Verifiable();
        actionMock.SetupGet(a => a.DisplayName).Returns(actionDisplayName);

        await actionsVM.Execute.Execute(actionMock.Object);

        Assert.Equal(actionDisplayName, actionsVM.LastDone);

        await actionsVM.Undo.Execute();

        await Task.Delay(1); // without this the variables do not get updated for some reason.

        Assert.Equal(actionDisplayName, actionsVM.LastUndone);
        Assert.NotEqual(actionDisplayName, actionsVM.LastDone);

        await actionsVM.Redo.Execute();

        await Task.Delay(1); // without this the variables do not get updated for some reason.

        Assert.Equal(actionDisplayName, actionsVM.LastDone);
        Assert.NotEqual(actionDisplayName, actionsVM.LastUndone);

        actionMock.Verify(a => a.Act(), Times.Exactly(2));
        actionMock.Verify(a => a.Revert(), Times.Once);

        // make sure clearing works
        await actionsVM.Clear.Execute();

        Assert.Null(actionsVM.LastDone);
        Assert.Null(actionsVM.LastUndone);
    }

    [Fact(DisplayName = "Notifies user of errors during acting, undo and redo")]
    [SuppressMessage("Globalization", "CA1303:Do not pass literals as localized parameters",
        Justification = "Unit tests do not require localization for exception messages.")]
    public async Task NotifiesUserOfErrors()
    {
        // configure an action that fails when executed
        var failingActMock = new Mock<IReversibleAction>();

        failingActMock.Setup(a => a.Act()).Throws(new Exception("Act doesn't work"));

        // configure an action that fails on reversion (on undo)
        var failingRevertMock = new Mock<IReversibleAction>();

        failingRevertMock.Setup(a => a.Revert()).Throws(new Exception("Revert doesn't work"));
        failingRevertMock.Setup(a => a.Act());

        // configure an action that fails on the second time being executed (on redo)
        var failingActOnUndoMock = new Mock<IReversibleAction>();

        var timesCalled = 0;

        failingActOnUndoMock.Setup(a => a.Revert());
        failingActOnUndoMock.Setup(a => a.Act()).Callback(() =>
            timesCalled = timesCalled switch
            {
                0 => 1,
                _ => throw new Exception("Act doesn't work")
            });

        var actionsVM = new ActionsViewModel();

        var timesFailureWasReported = 0;

        actionsVM.NotifyUserOfError.RegisterHandler(ic =>
        {
            timesFailureWasReported++;

            ic.SetOutput(Unit.Default);
        });

        // fails on execute
        await actionsVM.Execute.Execute(failingActMock.Object);

        // fails on undo
        await actionsVM.Execute.Execute(failingRevertMock.Object);
        await actionsVM.Undo.Execute();

        // fails on redo
        await actionsVM.Execute.Execute(failingActOnUndoMock.Object);
        await actionsVM.Undo.Execute();
        await actionsVM.Redo.Execute();

        Assert.Equal(3, timesFailureWasReported);
    }
}