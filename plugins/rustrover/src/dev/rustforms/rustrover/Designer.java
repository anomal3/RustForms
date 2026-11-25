package dev.rustforms.rustrover;

import com.intellij.execution.ExecutionException;
import com.intellij.execution.RunContentExecutor;
import com.intellij.execution.configurations.GeneralCommandLine;
import com.intellij.execution.process.ColoredProcessHandler;
import com.intellij.ide.util.PropertiesComponent;
import com.intellij.notification.NotificationGroupManager;
import com.intellij.notification.NotificationType;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.application.PathManager;
import com.intellij.openapi.util.SystemInfo;
import com.intellij.openapi.fileChooser.FileChooser;
import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory;
import com.intellij.openapi.fileEditor.FileDocumentManager;
import com.intellij.openapi.fileEditor.FileEditorManager;
import com.intellij.openapi.fileEditor.OpenFileDescriptor;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.LocalFileSystem;
import com.intellij.openapi.vfs.VfsUtil;
import com.intellij.openapi.vfs.VirtualFile;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/** Общее для действий плагина: где дизайнер, открытие форм, переход к коду, запуск. */
final class Designer {
    private static final String PATH_KEY = "rustforms.designer.path";

    private Designer() {
    }

    static void notify(Project project, String text, NotificationType type) {
        NotificationGroupManager.getInstance()
                .getNotificationGroup("RustForms")
                .createNotification(text, type)
                .notify(project);
    }

    /** Выбранный раньше путь, иначе записанный при сборке плагина; null — не найден. */
    static String knownExecutable() {
        String[] candidates = {PropertiesComponent.getInstance().getValue(PATH_KEY), builtInPath(), bundledPath()};
        for (String candidate : candidates) {
            if (candidate != null && Files.isRegularFile(Path.of(candidate))) {
                return candidate;
            }
        }
        return null;
    }

    /** То же, но если не найден — спросить. */
    static String executable(Project project) {
        String known = knownExecutable();
        if (known != null) {
            return known;
        }
        VirtualFile chosen = FileChooser.chooseFile(
                FileChooserDescriptorFactory.singleFile().withTitle("Где лежит rustforms-designer?"),
                project, null);
        if (chosen == null) {
            return null;
        }
        PropertiesComponent.getInstance().setValue(PATH_KEY, chosen.getPath());
        return chosen.getPath();
    }

    /** Дизайнер, вложенный в плагин из релиза: {@code rustforms/bin} рядом с {@code lib}. */
    private static String bundledPath() {
        String jar = PathManager.getJarPathForClass(Designer.class);
        if (jar == null) {
            return null;
        }
        Path plugin = Path.of(jar).getParent().getParent();
        String name = SystemInfo.isWindows ? "rustforms-designer.exe" : "rustforms-designer";
        return plugin.resolve("bin").resolve(name).toString();
    }

    private static String builtInPath() {
        try (InputStream in = Designer.class.getResourceAsStream("/designer.path")) {
            return in == null ? null : new String(in.readAllBytes(), StandardCharsets.UTF_8).strip();
        } catch (IOException e) {
            return null;
        }
    }

    /** Открывает форму во вкладке дизайнера. */
    static void open(Project project, VirtualFile form) {
        FileEditorManager editors = FileEditorManager.getInstance(project);
        editors.openFile(form, true, true);
        editors.setSelectedEditor(form, FormEditorProvider.ID);
    }

    static void showLine(Project project, Path file, int line) {
        VirtualFile target = LocalFileSystem.getInstance().refreshAndFindFileByNioFile(file);
        if (target == null) {
            return;
        }
        target.refresh(false, false);
        // Отступ тела метода — 8 пробелов.
        new OpenFileDescriptor(project, target, line - 1, 8).navigate(true);
    }

    /** Подхватить файлы, записанные дизайнером. */
    static void refresh(VirtualFile form) {
        VirtualFile directory = form.getParent();
        if (directory != null) {
            VfsUtil.markDirtyAndRefresh(true, false, true, directory);
        }
    }

    static void cargoRun(Project project, VirtualFile form) {
        VirtualFile root = form.getParent();
        while (root != null && root.findChild("Cargo.toml") == null) {
            root = root.getParent();
        }
        if (root == null) {
            notify(project, "Рядом с формой нет Cargo.toml", NotificationType.WARNING);
            return;
        }
        String directory = root.getPath();
        ApplicationManager.getApplication().invokeLater(() -> {
            FileDocumentManager.getInstance().saveAllDocuments();
            try {
                // Cargo пишет ход сборки в stderr, и IDE красила бы его целиком в красный.
                // Потоки объединяются, а цвета берутся из самого cargo.
                GeneralCommandLine cargo = new GeneralCommandLine("cargo", "run", "--color=always")
                        .withWorkDirectory(directory)
                        .withCharset(StandardCharsets.UTF_8)
                        .withRedirectErrorStream(true);
                new RunContentExecutor(project, new ColoredProcessHandler(cargo))
                        .withTitle("cargo run")
                        .withActivateToolWindow(true)
                        .run();
            } catch (ExecutionException e) {
                notify(project, "cargo не запустился: " + e.getMessage(), NotificationType.ERROR);
            }
        }, project.getDisposed());
    }

    /** Команда дизайнера без окна ({@code new}, {@code project}); возвращает путь к .rfrm. */
    static VirtualFile create(Project project, String... args) {
        String executable = executable(project);
        if (executable == null) {
            return null;
        }
        List<String> command = new ArrayList<>();
        command.add(executable);
        command.addAll(Arrays.asList(args));
        try {
            Process process = new ProcessBuilder(command).redirectErrorStream(true).start();
            String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).strip();
            if (process.waitFor() != 0) {
                notify(project, output, NotificationType.ERROR);
                return null;
            }
            return LocalFileSystem.getInstance().refreshAndFindFileByNioFile(Path.of(output));
        } catch (IOException e) {
            notify(project, "Дизайнер не запустился: " + e.getMessage(), NotificationType.ERROR);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
        return null;
    }
}
