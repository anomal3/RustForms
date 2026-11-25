package dev.rustforms.rustrover;

import com.intellij.openapi.fileEditor.FileEditor;
import com.intellij.openapi.fileEditor.FileEditorPolicy;
import com.intellij.openapi.fileEditor.FileEditorProvider;
import com.intellij.openapi.project.DumbAware;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.VirtualFile;
import org.jetbrains.annotations.NotNull;

/** Вкладка «Designer» у файлов .rfrm; рядом остаётся обычный текстовый редактор. */
public final class FormEditorProvider implements FileEditorProvider, DumbAware {
    static final String ID = "rustforms-designer";

    @Override
    public boolean accept(@NotNull Project project, @NotNull VirtualFile file) {
        return "rfrm".equals(file.getExtension()) && file.isInLocalFileSystem();
    }

    @Override
    public @NotNull FileEditor createEditor(@NotNull Project project, @NotNull VirtualFile file) {
        return new FormEditor(project, file);
    }

    @Override
    public @NotNull String getEditorTypeId() {
        return ID;
    }

    @Override
    public @NotNull FileEditorPolicy getPolicy() {
        return FileEditorPolicy.PLACE_BEFORE_DEFAULT_EDITOR;
    }
}
