using System;
using System.Runtime.InteropServices;
using Microsoft.VisualStudio;
using Microsoft.VisualStudio.Shell;
using Microsoft.VisualStudio.Shell.Interop;
using OleServiceProvider = Microsoft.VisualStudio.OLE.Interop.IServiceProvider;

namespace RustForms.VisualStudio
{
    [Guid("c3b0f7a9-1d24-4a58-b6e3-5f8c92d1a047")]
    public sealed class EditorFactory : IVsEditorFactory, IDisposable
    {
        private readonly IServiceProvider services;

        public EditorFactory(IServiceProvider services) => this.services = services;

        public int CreateEditorInstance(uint flags, string document, string physicalView, IVsHierarchy hierarchy,
            uint itemId, IntPtr existing, out IntPtr view, out IntPtr data, out string caption,
            out Guid commandUi, out int created)
        {
            ThreadHelper.ThrowIfNotOnUIThread();
            view = data = IntPtr.Zero;
            caption = "";
            commandUi = typeof(EditorFactory).GUID;
            created = 0;
            if ((flags & (VSConstants.CEF_OPENFILE | VSConstants.CEF_SILENT)) == 0)
            {
                return VSConstants.E_INVALIDARG;
            }
            // Файл уже открыт текстом: VS предложит закрыть и открыть заново.
            if (existing != IntPtr.Zero)
            {
                return VSConstants.VS_E_INCOMPATIBLEDOCDATA;
            }
            var pane = new EditorPane(services, document);
            view = Marshal.GetIUnknownForObject(pane);
            data = Marshal.GetIUnknownForObject(pane);
            return VSConstants.S_OK;
        }

        public int MapLogicalView(ref Guid logicalView, out string physicalView)
        {
            physicalView = null;
            return logicalView == VSConstants.LOGVIEWID.Primary_guid || logicalView == VSConstants.LOGVIEWID.Designer_guid
                ? VSConstants.S_OK
                : VSConstants.E_NOTIMPL;
        }

        public int SetSite(OleServiceProvider provider) => VSConstants.S_OK;

        public int Close() => VSConstants.S_OK;

        public void Dispose()
        {
        }
    }
}
