using CommunityToolkit.Mvvm.ComponentModel;
using ReactiveUI; // Add this for IActivatableViewModel
using System.Reactive.Disposables;

namespace ImageSort.Avalonia.ViewModels;

public class ViewModelBase : ObservableObject, IActivatableViewModel // Implement IActivatableViewModel
{
    // Add the required ViewModelActivator property
    public ViewModelActivator Activator { get; } = new ViewModelActivator();

    public ViewModelBase()
    {
        this.WhenActivated(disposables =>
        {
            // This is where you would typically put activation logic for the ViewModel
            // For now, it can be empty if there's no specific base activation logic.
            Disposable.Create(() => { /* Deactivation logic */ }).DisposeWith(disposables);
        });
    }
}
