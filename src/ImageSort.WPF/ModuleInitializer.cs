using System.Runtime.CompilerServices;
using ReactiveUI;
using ReactiveUI.Builder;

namespace ImageSort.WPF;

public static class ModuleInitializer
{
    [ModuleInitializer]
    public static void Initialize()
    {
        RxAppBuilder.CreateReactiveUIBuilder()
            .WithCoreServices()
            .WithPlatformServices()
            .BuildApp();
    }
}
