using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;

namespace RustForms.VisualStudio
{
    /// <summary>Общее: где дизайнер, сообщения, переход к коду, запуск.</summary>
    internal static class Tools
    {
        /// <summary>Переменная окружения RUSTFORMS_DESIGNER или путь, записанный при сборке.</summary>
        public static string Designer
        {
            get
            {
                var chosen = Environment.GetEnvironmentVariable("RUSTFORMS_DESIGNER");
                if (!string.IsNullOrEmpty(chosen) && File.Exists(chosen))
                {
                    return chosen;
                }
                var built = typeof(Tools).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>()
                    .FirstOrDefault(a => a.Key == "DesignerPath")?.Value;
                return built != null && File.Exists(built) ? built : null;
            }
        }

        public static void Message(IServiceProvider services, string text) =>
            VsShellUtilities.ShowMessageBox(services, text, "RustForms", OLEMSGICON.OLEMSGICON_INFO,
                OLEMSGBUTTON.OLEMSGBUTTON_OK, OLEMSGDEFBUTTON.OLEMSGDEFBUTTON_FIRST);

        /// <summary>Команда дизайнера без окна (new, project); возвращает путь к .rfrm или null.</summary>
        public static string Create(IServiceProvider services, params string[] arguments)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            var designer = Designer;
            if (designer == null)
            {
                Message(services, "Не найден rustforms-designer.exe: пересоберите расширение (build.ps1).");
                return null;
            }
            var info = new ProcessStartInfo(designer, string.Join(" ", arguments.Select(a => "\"" + a.TrimEnd('\\') + "\"")))
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                StandardOutputEncoding = Encoding.UTF8,
                StandardErrorEncoding = Encoding.UTF8,
            };
            using (var process = Process.Start(info))
            {
                var output = process.StandardOutput.ReadToEnd().Trim();
                var error = process.StandardError.ReadToEnd().Trim();
                process.WaitForExit();
                if (process.ExitCode == 0)
                {
                    return output;
                }
                Message(services, error);
                return null;
            }
        }

        /// <summary>Открыть форму: VS выберет для .rfrm наш редактор.</summary>
        public static void Open(IServiceProvider services, string path)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            VsShellUtilities.OpenDocument(services, path);
        }

        public static void ShowLine(IServiceProvider services, string path, int line)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            VsShellUtilities.OpenDocument(services, path, VSConstants.LOGVIEWID.TextView_guid,
                out _, out _, out IVsWindowFrame frame);
            frame?.Show();
            var view = frame == null ? null : VsShellUtilities.GetTextView(frame);
            if (view != null)
            {
                // Отступ тела метода — 8 пробелов.
                view.SetCaretPos(line - 1, 8);
                view.CenterLines(line - 1, 1);
            }
        }

        public static void SaveAll(IServiceProvider services)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            var table = (IVsRunningDocumentTable)services.GetService(typeof(SVsRunningDocumentTable));
            table?.SaveDocuments((uint)__VSRDTSAVEOPTIONS.RDTSAVEOPT_SaveIfDirty, null,
                (uint)VSConstants.VSITEMID.Nil, 0);
        }

        /// <summary>cargo run в отдельной консоли, которая остаётся открытой.</summary>
        public static void CargoRun(IServiceProvider services, string form)
        {
            var directory = new FileInfo(form).Directory;
            while (directory != null && !File.Exists(Path.Combine(directory.FullName, "Cargo.toml")))
            {
                directory = directory.Parent;
            }
            if (directory == null)
            {
                Message(services, "Рядом с формой нет Cargo.toml");
                return;
            }
            Process.Start(new ProcessStartInfo("cmd.exe", "/k cargo run") { WorkingDirectory = directory.FullName });
        }
    }
}
