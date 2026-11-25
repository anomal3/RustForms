package dev.rustforms.rustrover.ui;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.Objects;
import java.util.function.Consumer;
import javax.imageio.ImageIO;

/** Состояние открытой формы, общее для холста, Toolbox и Properties. */
public final class FormController {
    /** Что нужно от окружения (IDE или проверки). */
    public interface Host {
        /** Выполнить отправку команды, предварительно сохранив открытые файлы. */
        void beforeSend(Runnable send);

        /** Ответ на команду: ошибки, переход к коду, обновление файлов. */
        void replied(JsonObject reply);
    }

    private final Session session;
    private final Host host;
    private final List<Runnable> listeners = new ArrayList<>();
    private JsonObject state;
    private BufferedImage image;
    private String selected;
    private String tool;
    private double scale = 1.0;
    private int pending;

    public FormController(Session session, Host host) {
        this.session = session;
        this.host = host;
    }

    public void listen(Runnable listener) {
        listeners.add(listener);
    }

    private void changed() {
        listeners.forEach(Runnable::run);
    }

    public static JsonObject command(String op) {
        JsonObject request = new JsonObject();
        request.addProperty("op", op);
        return request;
    }

    public void send(JsonObject request) {
        send(request, reply -> {
        });
    }

    public void send(JsonObject request, Consumer<JsonObject> then) {
        request.addProperty("scale", scale);
        pending++;
        host.beforeSend(() -> session.send(request, reply -> {
            pending--;
            apply(reply);
            then.accept(reply);
            host.replied(reply);
        }));
    }

    /** Есть ли команды без ответа. */
    public boolean busy() {
        return pending > 0;
    }

    private void apply(JsonObject reply) {
        if (reply.has("state")) {
            state = reply.getAsJsonObject("state");
            JsonElement png = state.get("image");
            if (png != null && !png.isJsonNull()) {
                try {
                    image = ImageIO.read(new ByteArrayInputStream(Base64.getDecoder().decode(png.getAsString())));
                } catch (IOException e) {
                    image = null;
                }
            }
        }
        if (reply.has("select") && !reply.get("select").isJsonNull()) {
            selected = reply.get("select").getAsString();
        }
        if (selected != null && control(selected) == null) {
            selected = null;
        }
        changed();
    }

    public JsonObject state() {
        return state;
    }

    public JsonObject form() {
        return state == null ? null : state.getAsJsonObject("form");
    }

    public JsonArray controls() {
        return state == null ? new JsonArray() : state.getAsJsonArray("controls");
    }

    public JsonObject control(String name) {
        for (JsonElement element : controls()) {
            if (element.getAsJsonObject().get("name").getAsString().equals(name)) {
                return element.getAsJsonObject();
            }
        }
        return null;
    }

    public BufferedImage image() {
        return image;
    }

    public String selected() {
        return selected;
    }

    public void select(String name) {
        if (!Objects.equals(selected, name)) {
            selected = name;
            changed();
        }
    }

    /** Тип, который поставит следующий щелчок по форме. */
    public String tool() {
        return tool;
    }

    public void setTool(String kind) {
        if (!Objects.equals(tool, kind)) {
            tool = kind;
            changed();
        }
    }

    /** Масштаб экрана, под который нужна картинка. */
    public void setScale(double value) {
        if (value > 0 && Math.abs(value - scale) > 0.01) {
            scale = value;
            send(command("state"));
        }
    }

    public void add(String kind, double x, double y) {
        JsonObject request = command("add");
        request.addProperty("kind", kind);
        request.addProperty("x", x);
        request.addProperty("y", y);
        send(request);
    }

    private void onSelected(String op) {
        if (selected != null) {
            JsonObject request = command(op);
            request.addProperty("name", selected);
            send(request);
        }
    }

    public void delete() {
        onSelected("delete");
    }

    public void bringToFront() {
        onSelected("front");
    }

    public void sendToBack() {
        onSelected("back");
    }

    public void undo() {
        send(command("undo"));
    }

    public void redo() {
        send(command("redo"));
    }

    /** Создать (если нет) обработчик и перейти к нему; {@code name == null} — форма. */
    public void handler(String name, String event) {
        JsonObject request = command("handler");
        if (name != null) {
            request.addProperty("name", name);
        }
        if (event != null) {
            request.addProperty("event", event);
        }
        send(request);
    }

    public void copy(Consumer<String> clipboard) {
        if (selected != null) {
            JsonObject request = command("copy");
            request.addProperty("name", selected);
            send(request, reply -> {
                if (reply.has("text") && !reply.get("text").isJsonNull()) {
                    clipboard.accept(reply.get("text").getAsString());
                }
            });
        }
    }

    public void paste(String text) {
        JsonObject request = command("paste");
        request.addProperty("text", text);
        if (selected != null) {
            request.addProperty("name", selected);
        }
        send(request);
    }

    /** Изменить свойство контрола или формы ({@code name == null}). */
    public void set(String name, String property, Object value) {
        JsonObject request = command("set");
        if (name != null) {
            request.addProperty("name", name);
        }
        request.addProperty("property", property);
        if (value instanceof Boolean flag) {
            request.addProperty("value", flag);
        } else if (value instanceof Number number) {
            request.addProperty("value", number);
        } else {
            request.addProperty("value", value == null ? "" : value.toString());
        }
        send(request);
    }
}
