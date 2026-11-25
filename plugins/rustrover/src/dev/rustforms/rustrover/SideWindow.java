package dev.rustforms.rustrover;

import com.intellij.openapi.fileEditor.FileEditor;
import com.intellij.openapi.fileEditor.FileEditorManager;
import com.intellij.openapi.fileEditor.FileEditorManagerEvent;
import com.intellij.openapi.fileEditor.FileEditorManagerListener;
import com.intellij.openapi.project.DumbAware;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.wm.ToolWindow;
import com.intellij.openapi.wm.ToolWindowFactory;
import java.awt.BorderLayout;
import java.util.function.Function;
import javax.swing.JComponent;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.SwingConstants;
import org.jetbrains.annotations.NotNull;

/** Боковое окно IDE, показывающее часть активного дизайнера. */
abstract class SideWindow implements ToolWindowFactory, DumbAware {
    static final String TOOLBOX = "RustForms Toolbox";
    static final String PROPERTIES = "RustForms Properties";

    private final Function<FormEditor, JComponent> part;

    private SideWindow(Function<FormEditor, JComponent> part) {
        this.part = part;
    }

    public static final class Toolbox extends SideWindow {
        public Toolbox() {
            super(FormEditor::toolbox);
        }
    }

    public static final class Properties extends SideWindow {
        public Properties() {
            super(FormEditor::properties);
        }
    }

    @Override
    public void createToolWindowContent(@NotNull Project project, @NotNull ToolWindow window) {
        JPanel holder = new JPanel(new BorderLayout());
        var content = window.getContentManager().getFactory().createContent(holder, "", false);
        window.getContentManager().addContent(content);
        project.getMessageBus().connect(content).subscribe(
                FileEditorManagerListener.FILE_EDITOR_MANAGER, new FileEditorManagerListener() {
                    @Override
                    public void selectionChanged(@NotNull FileEditorManagerEvent event) {
                        show(holder, event.getNewEditor());
                    }
                });
        show(holder, FileEditorManager.getInstance(project).getSelectedEditor());
    }

    private void show(JPanel holder, FileEditor editor) {
        JComponent component = editor instanceof FormEditor form ? part.apply(form) : null;
        holder.removeAll();
        holder.add(component != null ? component
                : new JLabel("Откройте форму .rfrm", SwingConstants.CENTER));
        holder.revalidate();
        holder.repaint();
    }
}
