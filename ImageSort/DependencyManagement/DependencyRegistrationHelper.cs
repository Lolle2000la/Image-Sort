using ImageSort.FileSystem;
using Splat;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.DependencyManagement
{
    public static class DependencyRegistrationHelper
    {
        public static void RegisterManditoryDependencies(this IMutableDependencyResolver dependencyResolver)
        {
            dependencyResolver.Register<IFileSystem>(() => new FullAccessFileSystem());
        }
    }
}
