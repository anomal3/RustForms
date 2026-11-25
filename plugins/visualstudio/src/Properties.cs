using System.ComponentModel;
using System.Linq;
using Newtonsoft.Json.Linq;
using RustForms.VisualStudio.Designer;

namespace RustForms.VisualStudio
{
    // Объекты для окна Properties: VS сама строит таблицу по их свойствам и атрибутам.

    internal enum DockSide { None, Top, Bottom, Left, Right, Fill }

    internal abstract class PropertiesBase : ICustomTypeDescriptor
    {
        // Шапка окна Properties берёт отсюда имя и тип; остальное — обычное поведение.
        protected abstract string ComponentName { get; }
        protected abstract string ClassName { get; }

        string ICustomTypeDescriptor.GetComponentName() => ComponentName;
        string ICustomTypeDescriptor.GetClassName() => ClassName;
        AttributeCollection ICustomTypeDescriptor.GetAttributes() => TypeDescriptor.GetAttributes(this, true);
        TypeConverter ICustomTypeDescriptor.GetConverter() => TypeDescriptor.GetConverter(this, true);
        EventDescriptor ICustomTypeDescriptor.GetDefaultEvent() => null;
        PropertyDescriptor ICustomTypeDescriptor.GetDefaultProperty() => null;
        object ICustomTypeDescriptor.GetEditor(System.Type type) => TypeDescriptor.GetEditor(this, type, true);
        EventDescriptorCollection ICustomTypeDescriptor.GetEvents() => EventDescriptorCollection.Empty;
        EventDescriptorCollection ICustomTypeDescriptor.GetEvents(System.Attribute[] a) => EventDescriptorCollection.Empty;
        PropertyDescriptorCollection ICustomTypeDescriptor.GetProperties() => TypeDescriptor.GetProperties(this, true);
        PropertyDescriptorCollection ICustomTypeDescriptor.GetProperties(System.Attribute[] a) => TypeDescriptor.GetProperties(this, a, true);
        object ICustomTypeDescriptor.GetPropertyOwner(PropertyDescriptor property) => this;

        protected readonly FormController Controller;
        protected readonly JObject Data;
        protected readonly string Owner;

        protected PropertiesBase(FormController controller, JObject data, string owner)
        {
            Controller = controller;
            Data = data;
            Owner = owner;
        }

        protected string Get(string key) => (string)Data[key] ?? "";

        protected int Number(string key) => (int)(double)Data[key];

        protected bool Flag(string key) => (bool)Data[key];

        protected void Set(string property, object value) => Controller.Set(Owner, property, value);
    }

    internal sealed class FormPropertiesObject : PropertiesBase
    {
        public FormPropertiesObject(FormController controller)
            : base(controller, controller.Form, null)
        {
        }

        [Category("Форма"), DisplayName("Структура")]
        public string Name { get => Get("name"); set => Set("name", value); }

        [Category("Форма"), DisplayName("Заголовок")]
        public string Title { get => Get("title"); set => Set("title", value); }

        [Category("Расположение"), DisplayName("Ширина")]
        public int Width { get => Number("width"); set => Set("width", value); }

        [Category("Расположение"), DisplayName("Высота")]
        public int Height { get => Number("height"); set => Set("height", value); }

        [Category("Вид"), DisplayName("Фон (#rrggbb)")]
        public string BackColor { get => Get("back_color"); set => Set("back_color", value); }

        [Category("События"), DisplayName("load"), Description("Двойной клик по пустому месту формы создаёт обработчик.")]
        public string Load { get => Get("load"); set => Set("load", value); }

        protected override string ComponentName => Get("name");
        protected override string ClassName => "Форма";
    }

    internal class ControlPropertiesObject : PropertiesBase
    {
        public ControlPropertiesObject(FormController controller, JObject control)
            : base(controller, control, (string)control["name"])
        {
        }

        [Category("Контрол"), DisplayName("Имя")]
        public string Name { get => Get("name"); set => Set("name", value); }

        [Category("Контрол"), DisplayName("Свой тип"), Description("Путь к своему контролу, например crate::round_button::RoundButton.")]
        public string Custom { get => Get("custom"); set => Set("custom", value); }

        [Category("Расположение")]
        public int X { get => Number("x"); set => Set("x", value); }

        [Category("Расположение")]
        public int Y { get => Number("y"); set => Set("y", value); }

        [Category("Расположение"), DisplayName("Ширина")]
        public int Width { get => Number("width"); set => Set("width", value); }

        [Category("Расположение"), DisplayName("Высота")]
        public int Height { get => Number("height"); set => Set("height", value); }

        [Category("Расположение"), DisplayName("Dock")]
        public DockSide Dock
        {
            get => (DockSide)System.Enum.Parse(typeof(DockSide), Get("dock"), true);
            set => Set("dock", value.ToString().ToLowerInvariant());
        }

        private bool Anchored(string side) => ((JArray)Data["anchor"]).Any(s => (string)s == side);

        [Category("Расположение"), DisplayName("Якорь слева")]
        public bool AnchorLeft { get => Anchored("left"); set => Set("anchor:left", value); }

        [Category("Расположение"), DisplayName("Якорь сверху")]
        public bool AnchorTop { get => Anchored("top"); set => Set("anchor:top", value); }

        [Category("Расположение"), DisplayName("Якорь справа")]
        public bool AnchorRight { get => Anchored("right"); set => Set("anchor:right", value); }

        [Category("Расположение"), DisplayName("Якорь снизу")]
        public bool AnchorBottom { get => Anchored("bottom"); set => Set("anchor:bottom", value); }

        [Category("Поведение"), DisplayName("Видимый")]
        public bool Visible { get => Flag("visible"); set => Set("visible", value); }

        [Category("Поведение"), DisplayName("Доступен")]
        public bool Enabled { get => Flag("enabled"); set => Set("enabled", value); }

        protected string Handler(string name)
        {
            foreach (var item in (JArray)Data["events"])
            {
                if ((string)item["event"] == name)
                {
                    return (string)item["handler"] ?? "";
                }
            }
            return "";
        }

        protected void SetHandler(string name, string value) => Set("event:" + name, value);

        protected override string ComponentName => Get("name");
        protected override string ClassName => Get("kind");

        public static object Create(FormController controller, JObject control)
        {
            switch ((string)control["kind"])
            {
                case "Button": return new ButtonPropertiesObject(controller, control);
                case "Label": return new LabelPropertiesObject(controller, control);
                case "TextBox": return new TextBoxPropertiesObject(controller, control);
                case "CheckBox": return new CheckBoxPropertiesObject(controller, control);
                case "Panel": return new PanelPropertiesObject(controller, control);
                case "RadioButton": return new CheckBoxPropertiesObject(controller, control);
                case "ComboBox":
                case "ListBox": return new ListPropertiesObject(controller, control);
                case "ProgressBar": return new ProgressBarPropertiesObject(controller, control);
                case "MenuStrip": return new MenuStripPropertiesObject(controller, control);
                default: return new ControlPropertiesObject(controller, control);
            }
        }
    }

    internal sealed class ButtonPropertiesObject : ControlPropertiesObject
    {
        public ButtonPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("События"), DisplayName("click")]
        public string Click { get => Handler("click"); set => SetHandler("click", value); }
    }

    internal sealed class LabelPropertiesObject : ControlPropertiesObject
    {
        public LabelPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("Вид"), DisplayName("Цвет текста (#rrggbb)")]
        public string ForeColor { get => Get("fore_color"); set => Set("fore_color", value); }

        [Category("События"), DisplayName("click")]
        public string Click { get => Handler("click"); set => SetHandler("click", value); }
    }

    internal sealed class TextBoxPropertiesObject : ControlPropertiesObject
    {
        public TextBoxPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("Поведение"), DisplayName("Многострочный")]
        public bool Multiline { get => Flag("multiline"); set => Set("multiline", value); }

        [Category("События"), DisplayName("text_changed")]
        public string TextChanged { get => Handler("text_changed"); set => SetHandler("text_changed", value); }
    }

    internal sealed class CheckBoxPropertiesObject : ControlPropertiesObject
    {
        public CheckBoxPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("Поведение"), DisplayName("Отмечен")]
        public bool Checked { get => Flag("checked"); set => Set("checked", value); }

        [Category("События"), DisplayName("checked_changed")]
        public string CheckedChanged { get => Handler("checked_changed"); set => SetHandler("checked_changed", value); }

        [Category("События"), DisplayName("click")]
        public string Click { get => Handler("click"); set => SetHandler("click", value); }
    }

    internal sealed class ListPropertiesObject : ControlPropertiesObject
    {
        public ListPropertiesObject(FormController c, JObject o) : base(c, o) { }

        // Массив строк PropertyGrid редактирует своим окошком.
        [Category("Контрол"), DisplayName("Строки")]
        public string[] Items
        {
            get => ((JArray)Data["items"]).Select(i => (string)i).ToArray();
            set => Set("items", value);
        }

        [Category("Контрол"), DisplayName("Выбрана (с 0, -1 нет)")]
        public int Selected
        {
            get => (int?)Data["selected"] ?? -1;
            set => Set("selected", value);
        }

        [Category("События"), DisplayName("selected_changed")]
        public string SelectedChanged { get => Handler("selected_changed"); set => SetHandler("selected_changed", value); }
    }

    internal sealed class MenuStripPropertiesObject : ControlPropertiesObject
    {
        public MenuStripPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Пункты (Меню/Пункт)")]
        public string[] Items
        {
            get => ((JArray)Data["items"]).Select(i => (string)i).ToArray();
            set => Set("items", value);
        }

        [Category("События"), DisplayName("item_clicked")]
        public string ItemClicked { get => Handler("item_clicked"); set => SetHandler("item_clicked", value); }
    }

    internal sealed class ProgressBarPropertiesObject : ControlPropertiesObject
    {
        public ProgressBarPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Минимум")]
        public int Minimum { get => (int)Data["minimum"]; set => Set("minimum", value); }

        [Category("Контрол"), DisplayName("Максимум")]
        public int Maximum { get => (int)Data["maximum"]; set => Set("maximum", value); }

        [Category("Контрол"), DisplayName("Значение")]
        public int Value { get => (int)Data["value"]; set => Set("value", value); }
    }

    internal sealed class PanelPropertiesObject : ControlPropertiesObject
    {
        public PanelPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Вид"), DisplayName("Фон (#rrggbb)")]
        public string BackColor { get => Get("back_color"); set => Set("back_color", value); }

        [Category("Вид"), DisplayName("Рамка")]
        public bool Border { get => Flag("border"); set => Set("border", value); }
    }
}
