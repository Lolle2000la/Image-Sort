using System;
using System.Reactive.Disposables;

namespace ImageSort.Helpers;

/// <summary>
/// Extension methods for <see cref="IDisposable"/> to work with <see cref="CompositeDisposable"/>.
/// Replaces the DisposeWith method that was removed in ReactiveUI 23.x.
/// </summary>
public static class DisposableMixins
{
    public static T DisposeWith<T>(this T disposable, CompositeDisposable compositeDisposable) where T : IDisposable
    {
        compositeDisposable.Add(disposable);
        return disposable;
    }
}
