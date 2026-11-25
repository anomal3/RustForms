package dev.rustforms.rustrover.ui;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;
import javax.swing.SwingUtilities;

/** Процесс {@code rustforms-designer serve}: команда строкой JSON, ответ строкой JSON. */
public final class Session implements AutoCloseable {
    private final Process process;
    private final BufferedWriter input;
    private final BufferedReader output;
    private final ExecutorService worker = Executors.newSingleThreadExecutor(task -> {
        Thread thread = new Thread(task, "rustforms-session");
        thread.setDaemon(true);
        return thread;
    });

    public Session(String designer, Path form) throws IOException {
        process = new ProcessBuilder(designer, "serve", form.toString())
                .redirectError(ProcessBuilder.Redirect.DISCARD)
                .start();
        input = new BufferedWriter(new OutputStreamWriter(process.getOutputStream(), StandardCharsets.UTF_8));
        output = new BufferedReader(new InputStreamReader(process.getInputStream(), StandardCharsets.UTF_8));
    }

    /** Ответы приходят в поток интерфейса в том же порядке, что и команды. */
    public void send(JsonObject request, Consumer<JsonObject> reply) {
        if (worker.isShutdown()) {
            return;
        }
        String line = request.toString();
        worker.execute(() -> {
            JsonObject answer;
            try {
                input.write(line);
                input.write('\n');
                input.flush();
                String read = output.readLine();
                if (read == null) {
                    throw new IOException("дизайнер завершился");
                }
                answer = JsonParser.parseString(read).getAsJsonObject();
            } catch (IOException | RuntimeException e) {
                answer = new JsonObject();
                answer.addProperty("error", "Нет связи с дизайнером: " + e.getMessage());
            }
            JsonObject result = answer;
            SwingUtilities.invokeLater(() -> reply.accept(result));
        });
    }

    @Override
    public void close() {
        worker.shutdown();
        // Дизайнер выходит сам, когда закрывается его ввод.
        try {
            process.getOutputStream().close();
        } catch (IOException e) {
            process.destroy();
        }
    }
}
