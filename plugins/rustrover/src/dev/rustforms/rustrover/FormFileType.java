package dev.rustforms.rustrover;

import com.intellij.icons.AllIcons;
import com.intellij.openapi.fileTypes.FileType;
import javax.swing.Icon;
import org.jetbrains.annotations.NotNull;

public final class FormFileType implements FileType {
    public static final FormFileType INSTANCE = new FormFileType();

    private FormFileType() {
    }

    @Override
    public @NotNull String getName() {
        return "RustForms Form";
    }

    @Override
    public @NotNull String getDescription() {
        return "RustForms form";
    }

    @Override
    public @NotNull String getDefaultExtension() {
        return "rfrm";
    }

    @Override
    public Icon getIcon() {
        return AllIcons.FileTypes.UiForm;
    }

    @Override
    public boolean isBinary() {
        return false;
    }
}
