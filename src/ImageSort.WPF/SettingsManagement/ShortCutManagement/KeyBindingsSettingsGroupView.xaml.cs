using ImageSort.SettingsManagement;
using ReactiveUI;
using Splat;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Disposables;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace ImageSort.WPF.SettingsManagement.ShortCutManagement
{
    /// <summary>
    /// Interaction logic for KeyBindingsSettingsGroupView.xaml
    /// </summary>
    public partial class KeyBindingsSettingsGroupView : ReactiveUserControl<KeyBindingsSettingsGroupViewModel>
    {
        public KeyBindingsSettingsGroupView()
        {
            InitializeComponent();

            this.WhenActivated(disposableRegistration =>
            {
                ViewModel ??= Locator.Current.GetService<IEnumerable<SettingsGroupViewModelBase>>()
                    .OfType<KeyBindingsSettingsGroupViewModel>()
                    .FirstOrDefault();

                this.BindCommand(ViewModel,
                    vm => vm.RestoreDefaultBindings,
                    view => view.RestoreDefault)
                    .DisposeWith(disposableRegistration);

                // image management
                this.Bind(ViewModel,
                    vm => vm.Move,
                    view => view.Move.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.Delete,
                    view => view.Delete.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.Rename,
                    view => view.Rename.Hotkey)
                    .DisposeWith(disposableRegistration);

                // image selection
                this.Bind(ViewModel,
                    vm => vm.GoLeft,
                    view => view.GoLeft.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.GoRight,
                    view => view.GoRight.Hotkey)
                    .DisposeWith(disposableRegistration);

                // search images
                this.Bind(ViewModel,
                    vm => vm.SearchImages,
                    view => view.SearchImages.Hotkey)
                    .DisposeWith(disposableRegistration);

                // folder management
                this.Bind(ViewModel,
                    vm => vm.CreateFolder,
                    view => view.CreateFolder.Hotkey)
                    .DisposeWith(disposableRegistration);

                // folder opening
                this.Bind(ViewModel,
                    vm => vm.OpenFolder,
                    view => view.OpenFolder.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.OpenSelectedFolder,
                    view => view.OpenSelectedFolder.Hotkey)
                    .DisposeWith(disposableRegistration);

                // folder pinning
                this.Bind(ViewModel,
                    vm => vm.Pin,
                    view => view.Pin.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.PinSelected,
                    view => view.PinSelected.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.Unpin,
                    view => view.Unpin.Hotkey)
                    .DisposeWith(disposableRegistration);

                // folder selection
                this.Bind(ViewModel,
                    vm => vm.FolderUp,
                    view => view.FolderUp.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.FolderLeft,
                    view => view.FolderLeft.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.FolderDown,
                    view => view.FolderDown.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.FolderRight,
                    view => view.FolderRight.Hotkey)
                    .DisposeWith(disposableRegistration);

                // history
                this.Bind(ViewModel,
                    vm => vm.Undo,
                    view => view.Undo.Hotkey)
                    .DisposeWith(disposableRegistration);

                this.Bind(ViewModel,
                    vm => vm.Redo,
                    view => view.Redo.Hotkey)
                    .DisposeWith(disposableRegistration);
            });
        }
    }
}
