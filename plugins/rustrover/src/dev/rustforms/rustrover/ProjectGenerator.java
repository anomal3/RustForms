package dev.rustforms.rustrover;

import com.intellij.facet.ui.ValidationResult;
import com.intellij.icons.AllIcons;
import com.intellij.openapi.module.Module;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.startup.StartupManager;
import com.intellij.openapi.vfs.VirtualFile;
import com.intellij.platform.DirectoryProjectGeneratorBase;
import java.nio.file.Path;
import javax.swing.Icon;
import org.jetbrains.annotations.NotNull;

/** Пункт «RustForms» в окне New Project. */
public final class ProjectGenerator extends DirectoryProjectGeneratorBase<Object> {
    @Override
    public @NotNull String getName() {
        return "RustForms";
    }

    @Override
    public Icon getLogo() {
        return AllIcons.FileTypes.UiForm;
    }

    @Override
    public @NotNull ValidationResult validate(@NotNull String baseDirPath) {
        Path name = Path.of(baseDirPath).getFileName();
        return name != null && name.toString().matches("[A-Za-z][A-Za-z0-9_-]*")
                ? ValidationResult.OK
                : new ValidationResult("Имя проекта: латиница, цифры, - и _, начинается с буквы");
    }

    @Override
    public void generateProject(@NotNull Project project, @NotNull VirtualFile baseDir,
                                @NotNull Object settings, @NotNull Module module) {
        VirtualFile form = Designer.create(project, "project", baseDir.toNioPath().toString());
        if (form == null) {
            return;
        }
        baseDir.refresh(false, true);
        StartupManager.getInstance(project).runAfterOpened(() -> Designer.open(project, form));
    }
}
