﻿using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Text;
using Xunit;

namespace ImageSort.UnitTests.SettingsManagement
{
    public class SettingsGroupViewModelBaseTests
    {
        public class TestSettingsGroup : SettingsGroupViewModelBase
        {
            private bool _testProperty = false;

            public bool TestProperty
            {
                get => _testProperty;
                set => this.RaiseAndSetIfChanged(ref _testProperty, value);
            }

            private string _testString;

            public string TestString
            {
                get => _testString;
                set => this.RaiseAndSetIfChanged(ref _testString, value);
            }

            public override string Name => "TestGroup";

            public override string Header => "Test Group";
        }

        [Fact(DisplayName = "Saves changed properties in settings storage")]
        public void SavesChangedProperties()
        {
            var testSettingsGroup = new TestSettingsGroup();

            Assert.False(testSettingsGroup.SettingsStore.TryGetValue("TestProperty", out var _));
            Assert.False(testSettingsGroup.SettingsStore.TryGetValue("TestString", out var _));

            testSettingsGroup.TestProperty = true;
            testSettingsGroup.TestString = "first test value";

            Assert.True((bool)testSettingsGroup.SettingsStore["TestProperty"]);
            Assert.Equal("first test value", (string)testSettingsGroup.SettingsStore["TestString"]);

            testSettingsGroup.TestProperty = false;
            testSettingsGroup.TestString = "second test value";

            Assert.False((bool)testSettingsGroup.SettingsStore["TestProperty"]);
            Assert.Equal("second test value", (string)testSettingsGroup.SettingsStore["TestString"]);

            testSettingsGroup.TestString = null;

            Assert.Null(testSettingsGroup.SettingsStore["TestString"]);
        }
    }
}