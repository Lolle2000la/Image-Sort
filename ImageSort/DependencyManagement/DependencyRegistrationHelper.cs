using ImageSort.FileSystem;
using Splat;

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
