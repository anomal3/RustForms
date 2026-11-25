using System;
using System.Globalization;
using System.Linq;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Newtonsoft.Json.Linq;

namespace RustForms.VisualStudio.Designer
{
    /// <summary>Холст формы: выделение, перемещение, изменение размера, двойной клик.</summary>
    public sealed class FormCanvas : FrameworkElement
    {
        public const string DragFormat = "RustFormsKind";
        public const double Margin_ = 24;
        public const double TitleHeight = 30;
        private const double Grid = 8;
        private const double KnobSize = 8;
        private static readonly Brush Accent = new SolidColorBrush(Color.FromRgb(0, 120, 215));
        private static readonly int[][] AllSides =
        {
            new[] { -1, -1 }, new[] { 0, -1 }, new[] { 1, -1 }, new[] { -1, 0 },
            new[] { 1, 0 }, new[] { -1, 1 }, new[] { 0, 1 }, new[] { 1, 1 },
        };
        private static readonly int[][] FormSides = { new[] { 1, 0 }, new[] { 0, 1 }, new[] { 1, 1 } };

        private readonly FormController controller;

        // Перетаскивание: пока кнопка зажата, рисуется только рамка.
        private Point? pressed;
        private int[] sides;
        private Rect start;
        private Rect? ghost;
        private string dragged;

        public FormCanvas(FormController controller)
        {
            this.controller = controller;
            Focusable = true;
            FocusVisualStyle = null;
            AllowDrop = true;
            HorizontalAlignment = HorizontalAlignment.Left;
            VerticalAlignment = VerticalAlignment.Top;
            controller.Changed += () =>
            {
                InvalidateMeasure();
                InvalidateVisual();
            };
        }

        public bool Snap { get; set; } = true;

        public static Point Origin => new Point(Margin_, Margin_ + TitleHeight);

        private Size FormSize => controller.Form == null
            ? new Size(0, 0)
            : new Size((double)controller.Form["width"], (double)controller.Form["height"]);

        private Rect Client => new Rect(Origin, FormSize);

        protected override Size MeasureOverride(Size available) =>
            new Size(FormSize.Width + Margin_ * 2, FormSize.Height + Margin_ * 2 + TitleHeight);

        private double Snapped(double value, bool free) =>
            !Snap || free ? Math.Round(value) : Math.Round(value / Grid) * Grid;

        private static Rect FrameOf(JObject control)
        {
            var f = (JArray)control["frame"];
            return new Rect(Origin.X + (double)f[0], Origin.Y + (double)f[1],
                Math.Max(0, (double)f[2]), Math.Max(0, (double)f[3]));
        }

        private string Hit(Point p) => controller.Controls.Cast<JObject>().Reverse().Where(control =>
        {
            if ((bool)control["hidden"])
            {
                return false;
            }
            // Часть, выступающая за родителя, не видна и не кликается.
            for (var c = control; c != null; c = controller.Control((string)c["parent"]))
            {
                if (!FrameOf(c).Contains(p))
                {
                    return false;
                }
                if ((string)c["parent"] == null)
                {
                    break;
                }
            }
            return true;
        }).Select(c => (string)c["name"]).FirstOrDefault();

        private Rect? Selection
        {
            get
            {
                var control = controller.Selected == null ? null : controller.Control(controller.Selected);
                return control == null ? (Rect?)null : FrameOf(control);
            }
        }

        /// <summary>Щелчок по ярлычку вкладки переключает страницу.</summary>
        private void SwitchTab(string name, Point p)
        {
            var control = name == null ? null : controller.Control(name);
            if (!(control?["tabs"] is JArray edges))
            {
                return;
            }
            var frame = FrameOf(control);
            var x = p.X - frame.X;
            if (p.Y - frame.Y >= 28)
            {
                return;
            }
            for (var i = 0; i + 1 < edges.Count; i++)
            {
                if (x >= (double)edges[i] && x < (double)edges[i + 1])
                {
                    controller.Send("page", new JObject { ["name"] = name, ["index"] = i });
                }
            }
        }

        private static Rect Knob(Rect r, int[] side)
        {
            var x = side[0] < 0 ? r.Left : side[0] > 0 ? r.Right : r.Left + r.Width / 2;
            var y = side[1] < 0 ? r.Top : side[1] > 0 ? r.Bottom : r.Top + r.Height / 2;
            return new Rect(x - KnobSize / 2, y - KnobSize / 2, KnobSize, KnobSize);
        }

        private int[] KnobAt(Point p)
        {
            var selection = Selection;
            // У формы тянутся только правая и нижняя стороны.
            var target = selection ?? Client;
            return (selection != null ? AllSides : FormSides).FirstOrDefault(side =>
            {
                var knob = Knob(target, side);
                knob.Inflate(2, 2);
                return knob.Contains(p);
            });
        }

        private void Text(DrawingContext g, string text, Brush brush, Point at)
        {
            var formatted = new FormattedText(text, CultureInfo.CurrentUICulture, FlowDirection.LeftToRight,
                new Typeface("Segoe UI"), 13, brush, VisualTreeHelper.GetDpi(this).PixelsPerDip);
            g.DrawText(formatted, at);
        }

        protected override void OnRender(DrawingContext g)
        {
            // Прозрачная заливка, чтобы щелчки по пустому месту доходили до холста.
            g.DrawRectangle(Brushes.Transparent, null, new Rect(RenderSize));
            controller.SetScale(VisualTreeHelper.GetDpi(this).DpiScaleX);
            if (controller.Form == null)
            {
                Text(g, "Загрузка формы…", Brushes.Gray, new Point(Margin_, Margin_));
                return;
            }
            var client = Client;
            var title = new Rect(client.X, client.Y - TitleHeight, client.Width, TitleHeight);
            g.DrawRectangle(new SolidColorBrush(Color.FromRgb(32, 32, 40)), null, title);
            Text(g, (string)controller.Form["title"], Brushes.White, new Point(title.X + 10, title.Y + 6));
            if (controller.Image != null)
            {
                g.DrawImage(controller.Image, client);
            }
            foreach (JObject control in controller.Controls)
            {
                if ((bool)control["hidden"])
                {
                    continue;
                }
                var r = FrameOf(control);
                if (!(bool)control["visible"])
                {
                    g.DrawRectangle(new SolidColorBrush(Color.FromArgb(170, 255, 255, 255)), null, r);
                }
                var custom = (string)control["custom"];
                if (custom != null)
                {
                    Text(g, custom.Substring(custom.LastIndexOf(':') + 1), Accent, new Point(r.X, r.Y - 17));
                }
            }

            var selection = Selection;
            var pen = new Pen(Accent, 1);
            if (selection != null)
            {
                var outline = selection.Value;
                outline.Inflate(0.5, 0.5);
                g.DrawRectangle(null, pen, outline);
            }
            foreach (var side in selection != null ? AllSides : FormSides)
            {
                g.DrawRectangle(Brushes.White, pen, Knob(selection ?? client, side));
            }
            if (ghost != null)
            {
                g.DrawRectangle(null, new Pen(Accent, 1) { DashStyle = DashStyles.Dash }, ghost.Value);
            }
        }

        private static bool Free => (Keyboard.Modifiers & ModifierKeys.Alt) != 0;

        protected override void OnMouseLeftButtonDown(MouseButtonEventArgs e)
        {
            Focus();
            Press(e.GetPosition(this), e.ClickCount, Free);
            if (pressed != null)
            {
                CaptureMouse();
            }
            e.Handled = true;
        }

        protected override void OnMouseMove(MouseEventArgs e)
        {
            var p = e.GetPosition(this);
            if (pressed != null)
            {
                Drag(p, Free);
                return;
            }
            var side = controller.Form == null ? null : KnobAt(p);
            Cursor = side != null
                ? side[0] == 0 ? Cursors.SizeNS
                    : side[1] == 0 ? Cursors.SizeWE
                    : side[0] == side[1] ? Cursors.SizeNWSE : Cursors.SizeNESW
                : controller.Tool != null ? Cursors.Cross : Cursors.Arrow;
        }

        protected override void OnMouseLeftButtonUp(MouseButtonEventArgs e)
        {
            Release();
            ReleaseMouseCapture();
        }

        /// <summary>Нажатие кнопки мыши в координатах холста.</summary>
        public void Press(Point p, int clicks, bool free)
        {
            if (controller.Form == null)
            {
                return;
            }
            var tool = controller.Tool;
            if (tool != null)
            {
                if (Client.Contains(p))
                {
                    controller.Add(tool, Snapped(p.X - Origin.X, free), Snapped(p.Y - Origin.Y, free));
                    controller.Tool = null;
                }
                return;
            }
            if (clicks == 2)
            {
                pressed = null;
                if (Client.Contains(p))
                {
                    controller.Handler(Hit(p));
                }
                return;
            }
            var knob = KnobAt(p);
            if (knob == null)
            {
                var name = Hit(p);
                SwitchTab(name, p);
                controller.Select(name);
                if (name == null)
                {
                    return;
                }
                sides = new[] { 0, 0 };
            }
            else
            {
                sides = knob;
            }
            pressed = p;
            dragged = controller.Selected;
            start = dragged == null ? Client : Selection ?? Client;
        }

        /// <summary>Начало координат родителя перетаскиваемого контрола.</summary>
        private Point ParentOrigin
        {
            get
            {
                var control = dragged == null ? null : controller.Control(dragged);
                var parent = control == null ? null : (string)control["parent"];
                return parent == null ? Origin : FrameOf(controller.Control(parent)).TopLeft;
            }
        }

        public void Drag(Point p, bool free)
        {
            // Пока нет ответа на прошлую команду, положение на холсте устарело.
            if (pressed == null || controller.Busy)
            {
                return;
            }
            var dx = p.X - pressed.Value.X;
            var dy = p.Y - pressed.Value.Y;
            if (ghost == null && Math.Abs(dx) < 4 && Math.Abs(dy) < 4)
            {
                return;
            }
            var parent = ParentOrigin;
            var left = start.X - parent.X;
            var top = start.Y - parent.Y;
            var right = left + start.Width;
            var bottom = top + start.Height;
            if (dragged != null && sides[0] == 0 && sides[1] == 0)
            {
                left = Snapped(left + dx, free);
                top = Snapped(top + dy, free);
                right = left + start.Width;
                bottom = top + start.Height;
            }
            else
            {
                if (sides[0] < 0) left = Math.Min(Snapped(left + dx, free), right - KnobSize);
                else if (sides[0] > 0) right = Math.Max(Snapped(right + dx, free), left + KnobSize);
                if (sides[1] < 0) top = Math.Min(Snapped(top + dy, free), bottom - KnobSize);
                else if (sides[1] > 0) bottom = Math.Max(Snapped(bottom + dy, free), top + KnobSize);
            }
            ghost = dragged == null
                ? new Rect(Origin.X, Origin.Y, Math.Max(80, right - left), Math.Max(40, bottom - top))
                : new Rect(parent.X + left, parent.Y + top, Math.Max(0, right - left), Math.Max(0, bottom - top));
            InvalidateVisual();
        }

        public void Release()
        {
            if (ghost != null)
            {
                var g = ghost.Value;
                if (dragged == null)
                {
                    controller.Send("formSize", new JObject { ["width"] = g.Width, ["height"] = g.Height });
                }
                else
                {
                    var parent = ParentOrigin;
                    controller.Bounds(dragged, g.X - parent.X, g.Y - parent.Y, g.Width, g.Height,
                        sides[0] == 0 && sides[1] == 0);
                }
            }
            pressed = null;
            ghost = null;
            InvalidateVisual();
        }

        /// <summary>Стрелки: сдвиг на пиксель, с Shift — изменение размера.</summary>
        public void Nudge(int dx, int dy, bool resize)
        {
            var c = controller.Selected == null ? null : controller.Control(controller.Selected);
            if (c == null || controller.Busy)
            {
                return;
            }
            controller.Bounds(controller.Selected,
                (double)c["x"] + (resize ? 0 : dx), (double)c["y"] + (resize ? 0 : dy),
                (double)c["width"] + (resize ? dx : 0), (double)c["height"] + (resize ? dy : 0), false);
        }

        /// <summary>Escape: убрать инструмент или перейти к родителю.</summary>
        public void Cancel()
        {
            if (controller.Tool != null)
            {
                controller.Tool = null;
                return;
            }
            var c = controller.Selected == null ? null : controller.Control(controller.Selected);
            controller.Select((string)c?["parent"]);
        }

        protected override void OnDragOver(DragEventArgs e)
        {
            e.Effects = e.Data.GetDataPresent(DragFormat) ? DragDropEffects.Copy : DragDropEffects.None;
            e.Handled = true;
        }

        protected override void OnDrop(DragEventArgs e)
        {
            var p = e.GetPosition(this);
            if (e.Data.GetData(DragFormat) is string kind && Client.Contains(p))
            {
                controller.Add(kind, Snapped(p.X - Origin.X, false), Snapped(p.Y - Origin.Y, false));
                controller.Tool = null;
            }
            e.Handled = true;
        }
    }
}
