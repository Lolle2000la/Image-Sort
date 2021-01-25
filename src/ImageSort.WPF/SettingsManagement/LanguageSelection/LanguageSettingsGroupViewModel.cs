using ImageSort.Localization;
using ImageSort.SettingsManagement;
using ReactiveUI;
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Reactive;
using System.Reactive.Linq;
using System.Resources;
using System.Text;
using System.Threading.Tasks;

namespace ImageSort.WPF.SettingsManagement.LanguageSelection
{
    public class LanguageSettingsGroupViewModel : SettingsGroupViewModelBase
    {

        public LanguageSettingsGroupViewModel()
        {
            var resourceManager = new ResourceManager(typeof(Text));

            AvailableLanguages = CultureInfo.GetCultures(CultureTypes.AllCultures)
                .Where(c => resourceManager.GetResourceSet(c, true, false) is not null
                    || c.Name == "en")
                .ToArray();

            this.WhenAnyValue(x => x.Language)
                .Where(l => l is not null)
                .Subscribe(l => CultureInfo.CurrentCulture = CultureInfo.GetCultureInfo(l));

            var canApply = this.WhenAnyValue(x => x.SelectedLanguage)
                .Select(l => l is not null && l.Name != Language);

            ApplyLanguage = ReactiveCommand.Create(() =>
            {
                Language = SelectedLanguage.Name;
            }, canApply);
        }

        public override string Name => "LanguageSettings";

        public override string Header => Text.Language;

        private string _language = CultureInfo.InstalledUICulture.Name;
        public string Language
        {
            get => _language;
            set => this.RaiseAndSetIfChanged(ref _language, value);
        }

        private CultureInfo _selectedLanguage;
        public CultureInfo SelectedLanguage
        {
            get => _selectedLanguage;
            set => this.RaiseAndSetIfChanged(ref _selectedLanguage, value);
        }

        internal IEnumerable<CultureInfo> AvailableLanguages { get; }

        public ReactiveCommand<Unit, Unit> ApplyLanguage { get; }
    }
}
