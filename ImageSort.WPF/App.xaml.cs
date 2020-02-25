using ImageSort.DependencyManagement;
using ImageSort.FileSystem;
using ImageSort.WPF.FileSystem;
using ReactiveUI;
using Splat;
using System.Reflection;
using System.Windows;

namespace ImageSort.WPF
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            Locator.CurrentMutable.RegisterViewsForViewModels(Assembly.GetCallingAssembly());
            Locator.CurrentMutable.RegisterManditoryDependencies();
            Locator.CurrentMutable.Register<IRecycleBin>(() => new RecycleBin());
        }
    }
}
