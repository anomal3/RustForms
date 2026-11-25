package dev.rustforms.rustrover;

import com.intellij.notification.NotificationType;
import com.intellij.openapi.actionSystem.ActionUpdateThread;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.actionSystem.CommonDataKeys;
import com.intellij.openapi.project.DumbAwareAction;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.ui.Messages;
import com.intellij.openapi.vfs.VirtualFile;
import org.jetbrains.annotations.NotNull;

public final class NewFormAction extends DumbAwareAction {
    private static VirtualFile directoryOf(AnActionEvent e) {
        VirtualFile file = e.getData(CommonDataKeys.VIRTUAL_FILE);
        if (file == null) {
            return null;
        }
        return file.isDirectory() ? file : file.getParent();
    }

    @Override
    public @NotNull ActionUpdateThread getActionUpdateThread() {
        return ActionUpdateThread.BGT;
    }

    @Override
    public void update(@NotNull AnActionEvent e) {
        e.getPresentation().setEnabledAndVisible(e.getProject() != null && directoryOf(e) != null);
    }

    @Override
    public void actionPerformed(@NotNull AnActionEvent e) {
        Project project = e.getProject();
        VirtualFile directory = directoryOf(e);
        if (project == null || directory == null) {
            return;
        }
        String name = Messages.showInputDialog(project, "Имя структуры формы:", "New RustForms Form",
                null, "SecondForm", null);
        if (name == null || name.isBlank()) {
            return;
        }
        VirtualFile form = Designer.create(project, "new", directory.toNioPath().toString(), name.strip());
        if (form == null) {
            return;
        }
        String module = form.getNameWithoutExtension();
        Designer.notify(project, "Форма создана. Добавьте <b>mod " + module + ";</b> в main.rs",
                NotificationType.INFORMATION);
        Designer.open(project, form);
    }
}
