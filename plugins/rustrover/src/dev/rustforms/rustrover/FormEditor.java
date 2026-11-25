package dev.rustforms.rustrover;

import com.google.gson.JsonObject;
import com.intellij.icons.AllIcons;
import com.intellij.openapi.actionSystem.ActionManager;
import com.intellij.openapi.actionSystem.ActionToolbar;
import com.intellij.openapi.actionSystem.ActionUpdateThread;
import com.intellij.openapi.actionSystem.AnAction;
import com.intellij.openapi.actionSystem.DefaultActionGroup;
import com.intellij.openapi.project.DumbAwareToggleAction;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.actionSystem.CustomShortcutSet;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.fileEditor.FileDocumentManager;
import com.intellij.openapi.fileEditor.FileEditor;
import com.intellij.openapi.fileEditor.FileEditorState;
import com.intellij.openapi.ide.CopyPasteManager;
import com.intellij.openapi.project.DumbAwareAction;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.util.SystemInfo;
import com.intellij.openapi.util.UserDataHolderBase;
import com.intellij.openapi.vfs.VirtualFile;
import com.intellij.openapi.wm.ToolWindow;
import com.intellij.openapi.wm.ToolWindowManager;
import com.intellij.ui.components.JBScrollPane;
import dev.rustforms.rustrover.ui.FormCanvas;
import dev.rustforms.rustrover.ui.FormController;
import dev.rustforms.rustrover.ui.PropertiesPanel;
import dev.rustforms.rustrover.ui.Session;
import dev.rustforms.rustrover.ui.ToolboxPanel;
import java.awt.BorderLayout;
import java.awt.datatransfer.DataFlavor;
import java.awt.datatransfer.StringSelection;
import java.awt.event.InputEvent;
import java.awt.event.KeyEvent;
import java.beans.PropertyChangeListener;
import java.io.IOException;
import java.util.function.BooleanSupplier;
import javax.swing.Icon;
import javax.swing.JComponent;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.KeyStroke;
import org.jetbrains.annotations.NotNull;

/** Вкладка «Designer»: холст формы; Toolbox и Properties показываются в боковых окнах IDE. */
final class FormEditor extends UserDataHolderBase implements FileEditor, FormController.Host {
    private final Project project;
    private final VirtualFile file;
    private final JPanel root = new JPanel(new BorderLayout());
    private final JLabel status = new JLabel(" ");
    private Session session;
    private FormController controller;
    private FormCanvas canvas;
    private JComponent toolbox;
    private JComponent properties;

    FormEditor(Project project, VirtualFile file) {
        this.project = project;
        this.file = file;
        String executable = Designer.knownExecutable();
        if (executable == null) {
            root.add(new JLabel("Не найден rustforms-designer: пересоберите плагин (build.ps1)"));
            return;
        }
        try {
            session = new Session(executable, file.toNioPath());
        } catch (IOException e) {
            root.add(new JLabel("Дизайнер не запустился: " + e.getMessage()));
            return;
        }
        controller = new FormController(session, this);
        canvas = new FormCanvas(controller);
        toolbox = new ToolboxPanel(controller);
        properties = new PropertiesPanel(controller);
        root.add(toolbar(), BorderLayout.NORTH);
        root.add(new JBScrollPane(canvas));
        root.add(status, BorderLayout.SOUTH);
        shortcuts();
        controller.send(FormController.command("state"));
    }

    /** Родная панель действий IDE: тема, значки и подсказки такие же, как в остальных редакторах. */
    private JComponent toolbar() {
        DefaultActionGroup group = new DefaultActionGroup();
        group.add(action("Отменить", AllIcons.Actions.Undo, () -> flag("canUndo"), controller::undo));
        group.add(action("Повторить", AllIcons.Actions.Redo, () -> flag("canRedo"), controller::redo));
        group.addSeparator();
        BooleanSupplier selected = () -> controller.selected() != null;
        group.add(action("Удалить", AllIcons.Actions.GC, selected, controller::delete));
        group.add(action("На передний план", AllIcons.Actions.MoveUp, selected, controller::bringToFront));
        group.add(action("На задний план", AllIcons.Actions.MoveDown, selected, controller::sendToBack));
        group.addSeparator();
        group.add(new DumbAwareToggleAction("Сетка", "Привязка к сетке; Alt временно отключает", AllIcons.Graph.Grid) {
            private boolean on = true;

            @Override
            public boolean isSelected(@NotNull AnActionEvent e) {
                return on;
            }

            @Override
            public void setSelected(@NotNull AnActionEvent e, boolean value) {
                on = value;
                canvas.setSnap(value);
            }

            @Override
            public @NotNull ActionUpdateThread getActionUpdateThread() {
                return ActionUpdateThread.EDT;
            }
        });
        group.addSeparator();
        group.add(action("Запустить", AllIcons.Actions.Execute, () -> true,
                () -> Designer.cargoRun(project, file)));
        ActionToolbar bar = ActionManager.getInstance().createActionToolbar("RustFormsDesigner", group, true);
        bar.setTargetComponent(root);
        controller.listen(bar::updateActionsAsync);
        return bar.getComponent();
    }

    private boolean flag(String name) {
        return controller.state() != null && controller.state().get(name).getAsBoolean();
    }

    private static AnAction action(String text, Icon icon, BooleanSupplier enabled, Runnable run) {
        return new DumbAwareAction(text, null, icon) {
            @Override
            public void actionPerformed(@NotNull AnActionEvent e) {
                run.run();
            }

            @Override
            public void update(@NotNull AnActionEvent e) {
                e.getPresentation().setEnabled(enabled.getAsBoolean());
            }

            @Override
            public @NotNull ActionUpdateThread getActionUpdateThread() {
                return ActionUpdateThread.EDT;
            }
        };
    }

    /** Клавиши холста должны опережать одноимённые действия IDE. */
    private void shortcuts() {
        int command = SystemInfo.isMac ? InputEvent.META_DOWN_MASK : InputEvent.CTRL_DOWN_MASK;
        int shift = InputEvent.SHIFT_DOWN_MASK;
        key(KeyEvent.VK_DELETE, 0, controller::delete);
        key(KeyEvent.VK_ESCAPE, 0, canvas::cancel);
        key(KeyEvent.VK_Z, command, controller::undo);
        key(KeyEvent.VK_Z, command | shift, controller::redo);
        key(KeyEvent.VK_Y, command, controller::redo);
        key(KeyEvent.VK_C, command, () -> controller.copy(this::toClipboard));
        key(KeyEvent.VK_X, command, () -> {
            controller.copy(this::toClipboard);
            controller.delete();
        });
        key(KeyEvent.VK_V, command, () -> {
            String text = CopyPasteManager.getInstance().getContents(DataFlavor.stringFlavor);
            if (text != null) {
                controller.paste(text);
            }
        });
        int[][] arrows = {{KeyEvent.VK_LEFT, -1, 0}, {KeyEvent.VK_RIGHT, 1, 0},
                {KeyEvent.VK_UP, 0, -1}, {KeyEvent.VK_DOWN, 0, 1}};
        for (int[] a : arrows) {
            key(a[0], 0, () -> canvas.nudge(a[1], a[2], false));
            key(a[0], shift, () -> canvas.nudge(a[1], a[2], true));
        }
    }

    private void toClipboard(String text) {
        CopyPasteManager.getInstance().setContents(new StringSelection(text));
    }

    private void key(int code, int modifiers, Runnable action) {
        new DumbAwareAction() {
            @Override
            public void actionPerformed(@NotNull AnActionEvent e) {
                action.run();
            }
        }.registerCustomShortcutSet(new CustomShortcutSet(KeyStroke.getKeyStroke(code, modifiers)), canvas, this);
    }

    @Override
    public void beforeSend(Runnable send) {
        // Дизайнер читает файлы с диска, поэтому сначала сохраняется всё открытое.
        ApplicationManager.getApplication().invokeLater(() -> {
            FileDocumentManager.getInstance().saveAllDocuments();
            send.run();
        }, project.getDisposed());
    }

    @Override
    public void replied(JsonObject reply) {
        status.setText(reply.has("error") ? reply.get("error").getAsString() : " ");
        // Ответ приходит из обычной очереди Swing, а работать с файлами IDE
        // можно только из её очереди, с правом записи.
        ApplicationManager.getApplication().invokeLater(() -> {
            Designer.refresh(file);
            if (reply.has("open")) {
                JsonObject open = reply.getAsJsonObject("open");
                Designer.showLine(project, java.nio.file.Path.of(open.get("file").getAsString()),
                        open.get("line").getAsInt());
            }
        }, project.getDisposed());
    }

    JComponent toolbox() {
        return toolbox;
    }

    JComponent properties() {
        return properties;
    }

    @Override
    public void selectNotify() {
        if (controller == null) {
            return;
        }
        // Файл формы могли изменить, пока вкладка была не на виду.
        controller.send(FormController.command("state"));
        ToolWindowManager manager = ToolWindowManager.getInstance(project);
        manager.invokeLater(() -> {
            for (String id : new String[] {SideWindow.TOOLBOX, SideWindow.PROPERTIES}) {
                ToolWindow window = manager.getToolWindow(id);
                if (window != null) {
                    window.show();
                }
            }
        });
    }

    @Override
    public @NotNull JComponent getComponent() {
        return root;
    }

    @Override
    public JComponent getPreferredFocusedComponent() {
        return canvas;
    }

    @Override
    public @NotNull String getName() {
        return "Designer";
    }

    @Override
    public @NotNull VirtualFile getFile() {
        return file;
    }

    @Override
    public void setState(@NotNull FileEditorState state) {
    }

    @Override
    public boolean isModified() {
        return false;
    }

    @Override
    public boolean isValid() {
        return file.isValid();
    }

    @Override
    public void addPropertyChangeListener(@NotNull PropertyChangeListener listener) {
    }

    @Override
    public void removePropertyChangeListener(@NotNull PropertyChangeListener listener) {
    }

    @Override
    public void dispose() {
        if (session != null) {
            session.close();
        }
    }
}
