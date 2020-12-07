using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.WPF.Views
{
    static class ViewHelper
    {
        public static IDisposable WaitForViewModel<TViewModel>(this IViewFor<TViewModel> view, Action<TViewModel> callback) where TViewModel : class
        {
            return view.WhenAnyValue(x => x.ViewModel)
                .Where(vm => vm != null)
                .Subscribe(callback);
        }
    }
}
