package dev.rustforms.rustrover;

import com.intellij.openapi.actionSystem.ActionUpdateThread;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.actionSystem.CommonDataKeys;
import com.intellij.openapi.project.DumbAwareAction;
import com.intellij.openapi.vfs.VirtualFile;
import org.jetbrains.annotations.NotNull;

public final class OpenDesignerAction extends DumbAwareAction {
    /** Сама форма или форма рядом с выбранным .rs. */
    private static VirtualFile formOf(AnActionEvent e) {
        VirtualFile file = e.getData(CommonDataKeys.VIRTUAL_FILE);
        if (file == null || file.isDirectory() || file.getParent() == null) {
            return null;
        }
        String name = file.getName();
        if (name.endsWith(".rfrm")) {
            return file;
        }
        if (!name.endsWith(".rs")) {
            return null;
        }
        String stem = name.substring(0, name.length() - 3);
        if (stem.endsWith(".designer")) {
            stem = stem.substring(0, stem.length() - ".designer".length());
        }
        return file.getParent().findChild(stem + ".rfrm");
    }

    @Override
    public @NotNull ActionUpdateThread getActionUpdateThread() {
        return ActionUpdateThread.BGT;
    }

    @Override
    public void update(@NotNull AnActionEvent e) {
        e.getPresentation().setEnabledAndVisible(e.getProject() != null && formOf(e) != null);
    }

    @Override
    public void actionPerformed(@NotNull AnActionEvent e) {
        VirtualFile form = formOf(e);
        if (e.getProject() != null && form != null) {
            Designer.open(e.getProject(), form);
        }
    }
}
