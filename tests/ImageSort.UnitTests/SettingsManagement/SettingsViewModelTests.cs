using System;
using System.Collections.Generic;
using System.Text;
using ImageSort.SettingsManagement;
using Moq;
using Xunit;

namespace ImageSort.UnitTests.SettingsManagement
{
    public class SettingsViewModelTests
    {
        private class FirstGroupMock : SettingsGroupViewModelBase
        {
            public override string Name => "FirstGroup";
            public override string Header => "First group";

            public FirstGroupMock()
            {
                SettingsStore["some_setting"] = "fake value 1";
            }
        }

        private class SecondGroupMock : SettingsGroupViewModelBase
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

            var settingsGroups = new SettingsGroupViewModelBase[]
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

        [Fact(DisplayName = "Correctly restores settings from a dictionary")]
        public void CorrectlyRestoresFromDictionary()
        {
            var firstGroup = new FirstGroupMock();
            var secondGroup = new SecondGroupMock();

            var settingsGroups = new SettingsGroupViewModelBase[]
            {
                firstGroup,
                secondGroup
            };

            var settingsVM = new SettingsViewModel(settingsGroups);

            const string fakeValue1 = "some other fake value 1";
            const string fakeValue2 = "some other fake value 2";

            settingsVM.RestoreFromDictionary(new Dictionary<string, Dictionary<string, object>>
            {
                {firstGroup.Name, new Dictionary<string, object>
                    {
                        {"some_setting", fakeValue1}
                    }
                },
                {secondGroup.Name, new Dictionary<string, object>
                    {
                        {"some_setting", fakeValue2}
                    } 
                }
            });

            Assert.Equal(fakeValue1, firstGroup.SettingsStore["some_setting"]);

            Assert.Equal(fakeValue2, secondGroup.SettingsStore["some_setting"]);
        }
    }
}
