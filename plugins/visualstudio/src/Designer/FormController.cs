using System;
using System.IO;
using System.Linq;
using System.Windows.Media.Imaging;
using Newtonsoft.Json.Linq;

namespace RustForms.VisualStudio.Designer
{
    /// <summary>Состояние открытой формы, общее для холста, Properties и Toolbox.</summary>
    public sealed class FormController
    {
        private readonly Session session;
        private readonly Action beforeSend;
        private string selected;
        private double scale = 1.0;
        private int pending;

        public FormController(Session session, Action beforeSend)
        {
            this.session = session;
            this.beforeSend = beforeSend;
        }

        /// <summary>Изменились модель, выделение или инструмент.</summary>
        public event Action Changed;

        /// <summary>Пришёл ответ дизайнера: ошибки, переход к коду.</summary>
        public event Action<JObject> Replied;

        public JObject State { get; private set; }

        public BitmapImage Image { get; private set; }

        public JObject Form => (JObject)State?["form"];

        public JArray Controls => (JArray)State?["controls"] ?? new JArray();

        public JObject Control(string name) =>
            Controls.Cast<JObject>().FirstOrDefault(c => (string)c["name"] == name);

        /// <summary>Есть ли команды без ответа.</summary>
        public bool Busy => pending > 0;

        public string Selected => selected;

        public void Select(string name)
        {
            if (selected != name)
            {
                selected = name;
                Changed?.Invoke();
            }
        }

        /// <summary>Тип, который поставит следующий щелчок по форме.</summary>
        public string Tool { get; set; }

        public void SetScale(double value)
        {
            if (value > 0 && Math.Abs(value - scale) > 0.01)
            {
                scale = value;
                Send("state");
            }
        }

        public void Send(string op, JObject arguments = null, Action<JObject> then = null)
        {
            var request = arguments ?? new JObject();
            request["op"] = op;
            request["scale"] = scale;
            beforeSend();
            pending++;
            session.Send(request, reply =>
            {
                pending--;
                Apply(reply);
                then?.Invoke(reply);
                Replied?.Invoke(reply);
            });
        }

        private void Apply(JObject reply)
        {
            if (reply["state"] is JObject state)
            {
                State = state;
                var png = (string)state["image"];
                if (png != null)
                {
                    var image = new BitmapImage();
                    using (var stream = new MemoryStream(Convert.FromBase64String(png)))
                    {
                        image.BeginInit();
                        image.CacheOption = BitmapCacheOption.OnLoad;
                        image.StreamSource = stream;
                        image.EndInit();
                    }
                    image.Freeze();
                    Image = image;
                }
            }
            var select = (string)reply["select"];
            if (select != null)
            {
                selected = select;
            }
            if (selected != null && Control(selected) == null)
            {
                selected = null;
            }
            Changed?.Invoke();
        }

        public void Add(string kind, double x, double y) =>
            Send("add", new JObject { ["kind"] = kind, ["x"] = x, ["y"] = y });

        private void OnSelected(string op)
        {
            if (selected != null)
            {
                Send(op, new JObject { ["name"] = selected });
            }
        }

        public void Delete() => OnSelected("delete");

        public void BringToFront() => OnSelected("front");

        public void SendToBack() => OnSelected("back");

        public void Undo() => Send("undo");

        public void Redo() => Send("redo");

        /// <summary>Создать (если нет) обработчик и перейти к нему; name == null — форма.</summary>
        public void Handler(string name) =>
            Send("handler", name == null ? new JObject() : new JObject { ["name"] = name });

        public void Copy(Action<string> clipboard)
        {
            if (selected != null)
            {
                Send("copy", new JObject { ["name"] = selected }, reply =>
                {
                    var text = (string)reply["text"];
                    if (text != null)
                    {
                        clipboard(text);
                    }
                });
            }
        }

        public void Paste(string text)
        {
            var arguments = new JObject { ["text"] = text };
            if (selected != null)
            {
                arguments["name"] = selected;
            }
            Send("paste", arguments);
        }

        /// <summary>Изменить свойство контрола или формы (name == null).</summary>
        public void Set(string name, string property, object value)
        {
            var arguments = new JObject
            {
                ["property"] = property,
                ["value"] = value == null ? JValue.CreateString("") : JToken.FromObject(value),
            };
            if (name != null)
            {
                arguments["name"] = name;
            }
            Send("set", arguments);
        }

        public void Bounds(string name, double x, double y, double width, double height, bool moved) =>
            Send("bounds", new JObject
            {
                ["name"] = name, ["x"] = x, ["y"] = y,
                ["width"] = width, ["height"] = height, ["moved"] = moved,
            });
    }
}
