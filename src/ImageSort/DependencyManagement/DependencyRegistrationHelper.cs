using ImageSort.FileSystem;
using Splat;
using System.IO;

namespace ImageSort.DependencyManagement
{
    public static class DependencyRegistrationHelper
    {
        public static void RegisterManditoryDependencies(this IMutableDependencyResolver dependencyResolver)
        {
            dependencyResolver.Register<IFileSystem>(() => new FullAccessFileSystem());
            dependencyResolver.Register<FileSystemWatcher>(() => new FileSystemWatcher());
        }
    }
}
