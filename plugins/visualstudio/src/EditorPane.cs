using System;
using System.Collections;
using System.ComponentModel.Design;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.PlatformUI;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;
using Newtonsoft.Json.Linq;
using RustForms.VisualStudio.Designer;
using OleDataObjectInterface = Microsoft.VisualStudio.OLE.Interop.IDataObject;

namespace RustForms.VisualStudio
{
    /// <summary>Вкладка дизайнера. Каждое изменение сразу записано на диск, поэтому «грязной» она не бывает.</summary>
    internal sealed class EditorPane : WindowPane, IVsPersistDocData, IVsToolboxUser
    {
        private const string ToolboxTab = "RustForms";
        private static readonly string[] Kinds =
        {
            "Button", "Label", "LinkLabel", "TextBox", "NumericUpDown", "CheckBox", "RadioButton", "ComboBox", "ListBox", "TreeView", "DataGridView", "TrackBar", "ProgressBar", "PictureBox", "Panel", "GroupBox", "MenuStrip", "StatusStrip", "TabControl", "TabPage", "Timer", "DateTimePicker", "SplitContainer", "ToolStrip",
        };

        private readonly IServiceProvider package;
        private readonly string file;
        private readonly TextBlock status = new TextBlock { Margin = new Thickness(6, 2, 6, 2) };
        private Session session;
        private FormController controller;
        private FormCanvas canvas;
        private uint cookie;

        public EditorPane(IServiceProvider package, string file)
        {
            this.package = package;
            this.file = file;
        }

        protected override void Initialize()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            base.Initialize();
            var root = new DockPanel();
            root.SetResourceReference(Panel.BackgroundProperty, EnvironmentColors.ToolWindowBackgroundBrushKey);
            status.SetResourceReference(TextBlock.ForegroundProperty, EnvironmentColors.ToolWindowTextBrushKey);
            Content = root;

            var designer = Tools.Designer;
            if (designer == null)
            {
                status.Text = "Не найден rustforms-designer.exe: пересоберите расширение (build.ps1).";
                root.Children.Add(status);
                return;
            }
            session = new Session(designer, file);
            controller = new FormController(session, () =>
            {
                // Дизайнер читает файлы с диска, поэтому сначала сохраняется всё открытое.
                ThreadHelper.ThrowIfNotOnUIThread();
                Tools.SaveAll(package);
            });
            canvas = new FormCanvas(controller);

            var toolbar = Toolbar();
            DockPanel.SetDock(toolbar, Dock.Top);
            DockPanel.SetDock(status, Dock.Bottom);
            root.Children.Add(toolbar);
            root.Children.Add(status);
            root.Children.Add(new ScrollViewer
            {
                Content = canvas,
                HorizontalScrollBarVisibility = ScrollBarVisibility.Auto,
                VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
            });

            controller.Replied += OnReply;
            controller.Changed += ShowSelection;
            Commands();
            FillToolbox();
            controller.Send("state");
            ShowToolWindows();
        }

        /// <summary>Открывает Toolbox и Properties, не забирая фокус у формы.</summary>
        private void ShowToolWindows()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            if (!(GetService(typeof(SVsUIShell)) is IVsUIShell shell))
            {
                return;
            }
            foreach (var id in new[] { ToolWindowGuids80.Toolbox, ToolWindowGuids80.PropertiesWindow })
            {
                var guid = new Guid(id);
                if (ErrorHandler.Succeeded(shell.FindToolWindow((uint)__VSFINDTOOLWIN.FTW_fForceCreate, ref guid, out var frame)))
                {
                    frame?.ShowNoActivate();
                }
            }
        }

        private UIElement Toolbar()
        {
            var bar = new WrapPanel { Margin = new Thickness(4) };
            void Button(string text, Action action)
            {
                var button = new Button { Content = text, Margin = new Thickness(0, 0, 4, 0), Padding = new Thickness(10, 2, 10, 2) };
                button.SetResourceReference(FrameworkElement.StyleProperty, VsResourceKeys.ButtonStyleKey);
                button.Click += (s, e) => action();
                bar.Children.Add(button);
            }
            Button("Отменить", () => controller.Undo());
            Button("Повторить", () => controller.Redo());
            Button("Удалить", () => controller.Delete());
            Button("На передний план", () => controller.BringToFront());
            Button("На задний план", () => controller.SendToBack());
            var grid = new CheckBox { Content = "Сетка", IsChecked = true, Margin = new Thickness(8, 0, 12, 0), VerticalAlignment = VerticalAlignment.Center };
            grid.SetResourceReference(FrameworkElement.StyleProperty, VsResourceKeys.CheckBoxStyleKey);
            grid.Click += (s, e) => canvas.Snap = grid.IsChecked == true;
            bar.Children.Add(grid);
            Button("▶ Запустить", () => Tools.CargoRun(package, file));
            return bar;
        }

        /// <summary>Delete, стрелки, отмена и буфер обмена приходят командами VS, а не клавишами.</summary>
        private void Commands()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            var menu = (OleMenuCommandService)GetService(typeof(IMenuCommandService));
            void Add(Guid set, int id, Action action) =>
                menu.AddCommand(new MenuCommand((s, e) => action(), new CommandID(set, id)));

            var std97 = VSConstants.GUID_VSStandardCommandSet97;
            Add(std97, (int)VSConstants.VSStd97CmdID.Delete, () => controller.Delete());
            Add(std97, (int)VSConstants.VSStd97CmdID.Undo, () => controller.Undo());
            Add(std97, (int)VSConstants.VSStd97CmdID.Redo, () => controller.Redo());
            Add(std97, (int)VSConstants.VSStd97CmdID.Copy, () => controller.Copy(Clipboard.SetText));
            Add(std97, (int)VSConstants.VSStd97CmdID.Cut, () =>
            {
                controller.Copy(Clipboard.SetText);
                controller.Delete();
            });
            Add(std97, (int)VSConstants.VSStd97CmdID.Paste, () =>
            {
                if (Clipboard.ContainsText())
                {
                    controller.Paste(Clipboard.GetText());
                }
            });

            var std2k = VSConstants.VSStd2K;
            Add(std2k, (int)VSConstants.VSStd2KCmdID.DELETE, () => controller.Delete());
            Add(std2k, (int)VSConstants.VSStd2KCmdID.CANCEL, () => canvas.Cancel());
            var arrows = new[]
            {
                Tuple.Create(VSConstants.VSStd2KCmdID.LEFT, VSConstants.VSStd2KCmdID.LEFT_EXT, -1, 0),
                Tuple.Create(VSConstants.VSStd2KCmdID.RIGHT, VSConstants.VSStd2KCmdID.RIGHT_EXT, 1, 0),
                Tuple.Create(VSConstants.VSStd2KCmdID.UP, VSConstants.VSStd2KCmdID.UP_EXT, 0, -1),
                Tuple.Create(VSConstants.VSStd2KCmdID.DOWN, VSConstants.VSStd2KCmdID.DOWN_EXT, 0, 1),
            };
            foreach (var arrow in arrows)
            {
                Add(std2k, (int)arrow.Item1, () => canvas.Nudge(arrow.Item3, arrow.Item4, false));
                Add(std2k, (int)arrow.Item2, () => canvas.Nudge(arrow.Item3, arrow.Item4, true));
            }
        }

        private void OnReply(JObject reply)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            status.Text = (string)reply["error"] ?? "";
            if (reply["open"] is JObject open)
            {
                Tools.ShowLine(package, (string)open["file"], (int)open["line"]);
            }
        }

        /// <summary>Передаёт выделенное в окно Properties.</summary>
        private void ShowSelection()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            if (controller.Form == null || !(GetService(typeof(STrackSelection)) is ITrackSelection track))
            {
                return;
            }
            var form = new FormPropertiesObject(controller);
            var all = new ArrayList { form };
            object current = form;
            foreach (JObject control in controller.Controls)
            {
                var item = ControlPropertiesObject.Create(controller, control);
                all.Add(item);
                if ((string)control["name"] == controller.Selected)
                {
                    current = item;
                }
            }
            var container = new SelectionContainer(true, false)
            {
                SelectableObjects = all,
                SelectedObjects = new ArrayList { current },
            };
            // Выбор из списка в самом окне Properties.
            container.SelectedObjectsChanged += (s, e) =>
            {
                var chosen = container.SelectedObjects.Cast<object>().FirstOrDefault();
                controller.Select((chosen as ControlPropertiesObject)?.Name);
            };
            track.OnSelectChange(container);
        }

        // ---- Toolbox ----

        /// <summary>16×16 для Toolbox: прозрачность он понимает только через цвет-ключ.</summary>
        private static IntPtr Icon(string kind)
        {
            using (var stream = typeof(EditorPane).Assembly.GetManifestResourceStream("icons." + kind + ".png"))
            {
                if (stream == null)
                {
                    return IntPtr.Zero;
                }
                using (var source = new System.Drawing.Bitmap(stream))
                using (var small = new System.Drawing.Bitmap(source, 16, 16))
                {
                    for (var x = 0; x < 16; x++)
                    {
                        for (var y = 0; y < 16; y++)
                        {
                            var c = small.GetPixel(x, y);
                            small.SetPixel(x, y, c.A < 110
                                ? System.Drawing.Color.Magenta
                                : System.Drawing.Color.FromArgb(c.R, c.G, c.B));
                        }
                    }
                    return small.GetHbitmap();
                }
            }
        }

        private static OleDataObject ToolData(string kind)
        {
            var data = new OleDataObject();
            data.SetData(FormCanvas.DragFormat, kind);
            return data;
        }

        private void FillToolbox()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            if (!(GetService(typeof(SVsToolbox)) is IVsToolbox2 toolbox))
            {
                return;
            }
            // Элементы VS запоминает сама, поэтому вкладка пересоздаётся без повторов.
            toolbox.RemoveTab(ToolboxTab);
            toolbox.AddTab(ToolboxTab);
            foreach (var kind in Kinds)
            {
                var info = new[] { new TBXITEMINFO { bstrText = kind, dwFlags = (uint)__TBXITEMINFOFLAGS.TBXIF_DONTPERSIST } };
                var bitmap = Icon(kind);
                if (bitmap != IntPtr.Zero)
                {
                    info[0].hBmp = bitmap;
                    info[0].clrTransparent = 0x00FF00FF; // пурпурный = прозрачно
                }
                toolbox.AddItem(ToolData(kind), info, ToolboxTab);
            }
            toolbox.UpdateToolboxUI();
        }

        private static string KindOf(OleDataObjectInterface data) =>
            new OleDataObject(data).GetData(FormCanvas.DragFormat) as string;

        public int IsSupported(OleDataObjectInterface data) =>
            KindOf(data) != null ? VSConstants.S_OK : VSConstants.S_FALSE;

        /// <summary>Двойной клик по элементу Toolbox: следующий щелчок по форме поставит его.</summary>
        public int ItemPicked(OleDataObjectInterface data)
        {
            controller.Tool = KindOf(data);
            return VSConstants.S_OK;
        }

        // ---- Документ ----

        public int GetGuidEditorType(out Guid type)
        {
            type = typeof(EditorFactory).GUID;
            return VSConstants.S_OK;
        }

        public int IsDocDataDirty(out int dirty)
        {
            dirty = 0;
            return VSConstants.S_OK;
        }

        public int SetUntitledDocPath(string path) => VSConstants.S_OK;

        public int LoadDocData(string path) => VSConstants.S_OK;

        public int SaveDocData(VSSAVEFLAGS flags, out string newPath, out int canceled)
        {
            newPath = file;
            canceled = 0;
            return VSConstants.S_OK;
        }

        public int Close() => VSConstants.S_OK;

        public int OnRegisterDocData(uint docCookie, IVsHierarchy hierarchy, uint itemId)
        {
            cookie = docCookie;
            return VSConstants.S_OK;
        }

        public int RenameDocData(uint attributes, IVsHierarchy hierarchy, uint itemId, string path) => VSConstants.S_OK;

        public int IsDocDataReloadable(out int reloadable)
        {
            reloadable = 1;
            return VSConstants.S_OK;
        }

        public int ReloadDocData(uint flags)
        {
            controller?.Send("state");
            return VSConstants.S_OK;
        }

        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                session?.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
