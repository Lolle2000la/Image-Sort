using System;
using System.Collections.Generic;
using System.Text;
using ImageSort.SettingsManagement;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class SettingsViewModelTests
    {
        public class FirstGroupMock : SettingsGroupViewModelBase
        {
            public override string Header => "First group";

            public FirstGroupMock()
            {
                SettingsStore["some_setting"] = "fake value";
            }
        }

        public class SecondGroupMock : SettingsGroupViewModelBase
        {
            public override string Header => "Second group";

            public SecondGroupMock()
            {
                SettingsStore["some_setting"] = "fake value";
            }
        }

        [Fact(DisplayName = "Can retrieve a specific settings group")]
        public void CanRetrieveSpecificSettingsGroup()
        {
            var firstGroup = new FirstGroupMock();
            var secondGroup = new SecondGroupMock();

            var settingsGroups = new SettingsGroupViewModelBase []
            {
                firstGroup,
                secondGroup
            };

            var settingsVM = new SettingsViewModel(settingsGroups);

            Assert.Equal(firstGroup, settingsVM.GetGroup<FirstGroupMock>());
            Assert.Equal(secondGroup, settingsVM.GetGroup<SecondGroupMock>());
        }
    }
}
