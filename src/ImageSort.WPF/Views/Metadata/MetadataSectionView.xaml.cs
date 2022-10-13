using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace ImageSort.WPF.Views.Metadata;

/// <summary>
/// Interaction logic for MetadataSection.xaml
/// </summary>
public partial class MetadataSectionView : UserControl
{
    public string Title
    {
        get { return (string)GetValue(TitleProperty); }
        set { SetValue(TitleProperty, value); }
    }

    // Using a DependencyProperty as the backing store for Title.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty TitleProperty =
        DependencyProperty.Register("Title", typeof(string), typeof(MetadataSectionView), new PropertyMetadata("Missing dictionary tile"));
    
    public Dictionary<string, string> Fields
    {
        get { return (Dictionary<string, string>)GetValue(FieldsProperty); }
        set { SetValue(FieldsProperty, value); }
    }

    // Using a DependencyProperty as the backing store for Fields.  This enables animation, styling, binding, etc...
    public static readonly DependencyProperty FieldsProperty =
        DependencyProperty.Register("Fields", typeof(Dictionary<string, string>), typeof(MetadataSectionView), new PropertyMetadata(new Dictionary<string, string>() { { "", "" } }));
    
    public MetadataSectionView()
    {
        InitializeComponent();

        DataContext = this;
    }
}
