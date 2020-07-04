using Xunit;

namespace ImageSort.WPF.UiTests
{
    [CollectionDefinition("App collection", DisableParallelization = true)]
    public class AppCollection : ICollectionFixture<AppFixture>
    {

    }
}
