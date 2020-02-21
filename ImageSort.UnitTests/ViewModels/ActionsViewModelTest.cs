using System;
using System.Collections.Generic;
using System.Text;
using ImageSort.ViewModels;
using ImageSort.Actions;
using Xunit;
using Moq;
using System.Reactive.Linq;
using System.Threading.Tasks;

namespace ImageSort.UnitTests.ViewModels
{
    public class ActionsViewModelTest
    {
        [Fact(DisplayName = "Executes an action, adds it to the history, allows it to be undone and allows it to be redone and makes the last un-/done visible.")]
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

            Assert.Equal(actionDisplayName, actionsVM.LastUndone);
            Assert.NotEqual(actionDisplayName, actionsVM.LastDone);

            await actionsVM.Redo.Execute();

            Assert.Equal(actionDisplayName, actionsVM.LastDone);
            Assert.NotEqual(actionDisplayName, actionsVM.LastUndone);

            actionMock.Verify(a => a.Act(), Times.Exactly(2));
            actionMock.Verify(a => a.Revert(), Times.Once);
        }
    }
}
