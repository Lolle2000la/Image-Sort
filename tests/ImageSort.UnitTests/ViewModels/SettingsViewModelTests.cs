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
            public override string Name => "FirstGroup";
            public override string Header => "First group";

            public FirstGroupMock()
            {
                SettingsStore["some_setting"] = "fake value 1";
            }
        }

        public class SecondGroupMock : SettingsGroupViewModelBase
        {
            public override string Name => "SecondGroup";
            public override string Header => "Second group";

            public SecondGroupMock()
            {
                SettingsStore["some_setting"] = "fake value 2";
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

        [Fact(DisplayName = "Correctly converts settings into a dictionary")]
        public void CorrectlyConvertsToDictionary()
        {
            var firstGroup = new FirstGroupMock();
            var secondGroup = new SecondGroupMock();

            var settingsGroups = new SettingsGroupViewModelBase[]
            {
                firstGroup,
                secondGroup
            };

            var settingsVM = new SettingsViewModel(settingsGroups);

            var settingsDict = settingsVM.AsDictionary();

            Assert.Equal("fake value 2", settingsDict[secondGroup.Name]["some_setting"]);

            Assert.Equal("fake value 1", settingsDict[firstGroup.Name]["some_setting"]);
        }
    }
}
