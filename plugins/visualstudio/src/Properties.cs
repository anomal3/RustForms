using System.ComponentModel;
using System.Linq;
using Newtonsoft.Json.Linq;
using RustForms.VisualStudio.Designer;

namespace RustForms.VisualStudio
{
    // Объекты для окна Properties: VS сама строит таблицу по их свойствам и атрибутам.

    [System.AttributeUsage(System.AttributeTargets.Property)]
    internal sealed class ControlEventAttribute : System.Attribute
    {
        public ControlEventAttribute(string name) => Event = name;

        public string Event { get; }
    }

    internal enum TextAlignment { Default, Left, Center, Right }

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
        PropertyDescriptorCollection ICustomTypeDescriptor.GetProperties() =>
            Shown(TypeDescriptor.GetProperties(this, true));
        PropertyDescriptorCollection ICustomTypeDescriptor.GetProperties(System.Attribute[] a) =>
            Shown(TypeDescriptor.GetProperties(this, a, true));

        private PropertyDescriptorCollection Shown(PropertyDescriptorCollection all)
        {
            var events = (Data["events"] as JArray)?.Select(e => (string)e["event"]).ToList();
            return new PropertyDescriptorCollection(all.Cast<PropertyDescriptor>()
                .Where(p => !(p.Attributes[typeof(ControlEventAttribute)] is ControlEventAttribute e)
                    || events != null && events.Contains(e.Event))
                .ToArray());
        }
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

        [Category("События"), DisplayName("closing")]
        public string Closing { get => Get("closing"); set => Set("closing", value); }

        [Category("События"), DisplayName("resize")]
        public string Resize { get => Get("resize"); set => Set("resize", value); }

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

        [Category("Вид"), DisplayName("Цвет текста (#rrggbb)")]
        public string ForeColor { get => Get("fore_color"); set => Set("fore_color", value); }

        [Category("Вид"), DisplayName("Фон (#rrggbb)")]
        public string BackColor { get => Get("back_color"); set => Set("back_color", value); }

        [Category("Вид"), DisplayName("Размер шрифта (0 — обычный)")]
        public int FontSize { get => (int?)(double?)Data["font_size"] ?? 0; set => Set("font_size", value); }

        [Category("Вид"), DisplayName("Жирный")]
        public bool Bold { get => Flag("bold"); set => Set("bold", value); }

        [Category("Вид"), DisplayName("Выравнивание")]
        public TextAlignment Align
        {
            get => Data["align"].Type == JTokenType.Null ? TextAlignment.Default
                : (TextAlignment)System.Enum.Parse(typeof(TextAlignment), Get("align"), true);
            set => Set("align", value == TextAlignment.Default ? "" : value.ToString().ToLowerInvariant());
        }

        [Category("Поведение"), DisplayName("Подсказка")]
        public string Tooltip { get => Get("tooltip"); set => Set("tooltip", value); }

        [Category("Поведение"), DisplayName("Порядок Tab")]
        public int TabIndex { get => Number("tab_index"); set => Set("tab_index", value); }

        [ControlEvent("click"), Category("События"), DisplayName("click")]
        public string OnClick { get => Handler("click"); set => SetHandler("click", value); }

        [ControlEvent("text_changed"), Category("События"), DisplayName("text_changed")]
        public string OnTextChanged { get => Handler("text_changed"); set => SetHandler("text_changed", value); }

        [ControlEvent("checked_changed"), Category("События"), DisplayName("checked_changed")]
        public string OnCheckedChanged { get => Handler("checked_changed"); set => SetHandler("checked_changed", value); }

        [ControlEvent("selected_changed"), Category("События"), DisplayName("selected_changed")]
        public string OnSelectedChanged { get => Handler("selected_changed"); set => SetHandler("selected_changed", value); }

        [ControlEvent("item_clicked"), Category("События"), DisplayName("item_clicked")]
        public string OnItemClicked { get => Handler("item_clicked"); set => SetHandler("item_clicked", value); }

        [ControlEvent("double_click"), Category("События"), DisplayName("double_click")]
        public string OnDoubleClick { get => Handler("double_click"); set => SetHandler("double_click", value); }

        [ControlEvent("mouse_enter"), Category("События"), DisplayName("mouse_enter")]
        public string OnMouseEnter { get => Handler("mouse_enter"); set => SetHandler("mouse_enter", value); }

        [ControlEvent("mouse_leave"), Category("События"), DisplayName("mouse_leave")]
        public string OnMouseLeave { get => Handler("mouse_leave"); set => SetHandler("mouse_leave", value); }

        [ControlEvent("mouse_down"), Category("События"), DisplayName("mouse_down")]
        public string OnMouseDown { get => Handler("mouse_down"); set => SetHandler("mouse_down", value); }

        [ControlEvent("mouse_up"), Category("События"), DisplayName("mouse_up")]
        public string OnMouseUp { get => Handler("mouse_up"); set => SetHandler("mouse_up", value); }

        [ControlEvent("key_down"), Category("События"), DisplayName("key_down")]
        public string OnKeyDown { get => Handler("key_down"); set => SetHandler("key_down", value); }

        [ControlEvent("key_up"), Category("События"), DisplayName("key_up")]
        public string OnKeyUp { get => Handler("key_up"); set => SetHandler("key_up", value); }

        [ControlEvent("got_focus"), Category("События"), DisplayName("got_focus")]
        public string OnGotFocus { get => Handler("got_focus"); set => SetHandler("got_focus", value); }

        [ControlEvent("lost_focus"), Category("События"), DisplayName("lost_focus")]
        public string OnLostFocus { get => Handler("lost_focus"); set => SetHandler("lost_focus", value); }

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
    }

    internal sealed class LabelPropertiesObject : ControlPropertiesObject
    {
        public LabelPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }
    }

    internal sealed class TextBoxPropertiesObject : ControlPropertiesObject
    {
        public TextBoxPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("Поведение"), DisplayName("Многострочный")]
        public bool Multiline { get => Flag("multiline"); set => Set("multiline", value); }

        [Category("Поведение"), DisplayName("Только чтение")]
        public bool ReadOnly { get => Flag("read_only"); set => Set("read_only", value); }

        [Category("Поведение"), DisplayName("Пароль")]
        public bool Password { get => Flag("password"); set => Set("password", value); }
    }

    internal sealed class CheckBoxPropertiesObject : ControlPropertiesObject
    {
        public CheckBoxPropertiesObject(FormController c, JObject o) : base(c, o) { }

        [Category("Контрол"), DisplayName("Текст")]
        public string Text { get => Get("text"); set => Set("text", value); }

        [Category("Поведение"), DisplayName("Отмечен")]
        public bool Checked { get => Flag("checked"); set => Set("checked", value); }
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

        [Category("Вид"), DisplayName("Рамка")]
        public bool Border { get => Flag("border"); set => Set("border", value); }
    }
}
