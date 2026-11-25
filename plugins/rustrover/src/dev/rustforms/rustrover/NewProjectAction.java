package dev.rustforms.rustrover;

import com.intellij.ide.impl.ProjectUtil;
import com.intellij.openapi.actionSystem.ActionUpdateThread;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.fileChooser.FileChooser;
import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory;
import com.intellij.openapi.project.DumbAwareAction;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.VirtualFile;
import org.jetbrains.annotations.NotNull;

public final class NewProjectAction extends DumbAwareAction {
    @Override
    public @NotNull ActionUpdateThread getActionUpdateThread() {
        return ActionUpdateThread.BGT;
    }

    @Override
    public void update(@NotNull AnActionEvent e) {
        e.getPresentation().setEnabledAndVisible(e.getProject() != null);
    }

    @Override
    public void actionPerformed(@NotNull AnActionEvent e) {
        Project project = e.getProject();
        if (project == null) {
            return;
        }
        VirtualFile directory = FileChooser.chooseFile(
                FileChooserDescriptorFactory.singleDir()
                        .withTitle("Пустая папка нового проекта (имя латиницей)"),
                project, null);
        if (directory == null) {
            return;
        }
        if (Designer.create(project, "project", directory.toNioPath().toString()) != null) {
            ProjectUtil.openOrImport(directory.toNioPath(), project, false);
        }
    }
}
