using System;
using System.ComponentModel.Design;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;
using Task = System.Threading.Tasks.Task;

namespace RustForms.VisualStudio
{
    [PackageRegistration(UseManagedResourcesOnly = true, AllowsBackgroundLoading = true)]
    [Guid("7d1f3b52-6c1e-4c8a-9d57-2f0b8a3e91c4")]
    [ProvideMenuResource("Menus.ctmenu", 1)]
    [ProvideEditorFactory(typeof(EditorFactory), 110)]
    [ProvideEditorExtension(typeof(EditorFactory), ".rfrm", 100)]
    [ProvideEditorLogicalView(typeof(EditorFactory), VSConstants.LOGVIEWID.Designer_string)]
    public sealed class RustFormsPackage : AsyncPackage
    {
        private static readonly Guid Commands = new Guid("a4c9e0d1-35b7-4f62-8e1a-90d4b7c2f518");

        protected override async Task InitializeAsync(CancellationToken cancellationToken,
            IProgress<ServiceProgressData> progress)
        {
            await JoinableTaskFactory.SwitchToMainThreadAsync(cancellationToken);
            RegisterEditorFactory(new EditorFactory(this));
            var menu = (OleMenuCommandService)await GetServiceAsync(typeof(IMenuCommandService));
            menu.AddCommand(new MenuCommand((s, e) => NewProject(), new CommandID(Commands, 0x0100)));
            menu.AddCommand(new MenuCommand((s, e) => NewForm(), new CommandID(Commands, 0x0101)));
            menu.AddCommand(new MenuCommand((s, e) => OpenForm(), new CommandID(Commands, 0x0102)));
        }

        private void NewProject()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            using (var dialog = new System.Windows.Forms.FolderBrowserDialog
            {
                Description = "Пустая папка нового проекта (имя латиницей)",
            })
            {
                if (dialog.ShowDialog() != System.Windows.Forms.DialogResult.OK)
                {
                    return;
                }
                var form = Tools.Create(this, "project", dialog.SelectedPath);
                if (form != null)
                {
                    ((IVsSolution7)GetService(typeof(SVsSolution))).OpenFolder(dialog.SelectedPath);
                    Tools.Open(this, form);
                }
            }
        }

        private void NewForm()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            var dialog = new Microsoft.Win32.SaveFileDialog
            {
                Title = "Новая форма (например second_form)",
                Filter = "Форма RustForms|*.rfrm",
                FileName = "second_form.rfrm",
            };
            if (dialog.ShowDialog() != true)
            {
                return;
            }
            // second_form.rfrm -> структура SecondForm
            var name = string.Concat(Path.GetFileNameWithoutExtension(dialog.FileName)
                .Split(new[] { '_', '-', ' ' }, StringSplitOptions.RemoveEmptyEntries)
                .Select(word => char.ToUpperInvariant(word[0]) + word.Substring(1)));
            var form = Tools.Create(this, "new", Path.GetDirectoryName(dialog.FileName), name);
            if (form != null)
            {
                Tools.Open(this, form);
                Tools.Message(this, "Форма создана. Добавьте «mod " + Path.GetFileNameWithoutExtension(form) + ";» в main.rs.");
            }
        }

        private void OpenForm()
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            var dialog = new Microsoft.Win32.OpenFileDialog { Filter = "Форма RustForms|*.rfrm" };
            if (dialog.ShowDialog() == true)
            {
                Tools.Open(this, dialog.FileName);
            }
        }
    }
}
