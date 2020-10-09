using System;
using System.Collections.Generic;
using System.IO;
using ImageSort.FileSystem;
using ImageSort.SettingsManagement;
using Splat;

namespace ImageSort.DependencyManagement
{
    public static class DependencyRegistrationHelper
    {
        public static void RegisterManditoryDependencies(this IMutableDependencyResolver dependencyResolver)
        {
            dependencyResolver.Register<IFileSystem>(() => new FullAccessFileSystem());
            dependencyResolver.Register(() => new FileSystemWatcher());
        }

        /// <summary>
        ///     Registers settings and gives the possibility to registrate custom ones.
        /// </summary>
        /// <param name="registration">
        ///     Allows for registration of custom settings.
        ///     Simply add them to the <see cref="List{SettingsGroupViewModelBase}" /> and hold onto the added instances.
        /// </param>
        public static void RegisterSettings(this IMutableDependencyResolver dependencyResolver,
            Action<List<SettingsGroupViewModelBase>> registration = null)
        {
            var settings = new List<SettingsGroupViewModelBase>();

            var userSettings = new List<SettingsGroupViewModelBase>();
            registration?.Invoke(userSettings);

            if (userSettings.Count > 0) settings.AddRange(userSettings);

            dependencyResolver.RegisterConstant<IEnumerable<SettingsGroupViewModelBase>>(settings);
        }
    }
}