package dev.rustforms.rustrover.ui;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.RenderingHints;
import java.awt.datatransfer.DataFlavor;
import java.awt.event.InputEvent;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import javax.swing.JPanel;
import javax.swing.TransferHandler;

/** Холст формы: выделение, перемещение, изменение размера, двойной клик. */
public final class FormCanvas extends JPanel {
    public static final String DRAG_PREFIX = "rustforms:";
    public static final int MARGIN = 24;
    public static final int TITLE = 30;
    private static final int GRID = 8;
    private static final int KNOB = 8;
    private static final Color ACCENT = new Color(0, 120, 215);
    private static final int[][] SIDES = {
            {-1, -1}, {0, -1}, {1, -1}, {-1, 0}, {1, 0}, {-1, 1}, {0, 1}, {1, 1}};

    private final FormController controller;
    private boolean snap = true;

    /** Перетаскивание: пока кнопка зажата, рисуется только рамка. */
    private Point pressed;
    private int[] sides;
    private Rectangle start;
    private Rectangle ghost;
    private String dragged;

    public FormCanvas(FormController controller) {
        this.controller = controller;
        setFocusable(true);
        controller.listen(() -> {
            revalidate();
            repaint();
        });
        Mouse mouse = new Mouse();
        addMouseListener(mouse);
        addMouseMotionListener(mouse);
        setTransferHandler(new Drop());
    }

    public void setSnap(boolean value) {
        snap = value;
    }

    private int snapped(double value, int modifiers) {
        boolean free = !snap || (modifiers & InputEvent.ALT_DOWN_MASK) != 0;
        return (int) (free ? Math.round(value) : Math.round(value / GRID) * GRID);
    }

    /** Левый верхний угол клиентской области формы. */
    public static Point origin() {
        return new Point(MARGIN, MARGIN + TITLE);
    }

    private Dimension formSize() {
        JsonObject form = controller.form();
        return form == null ? new Dimension()
                : new Dimension(form.get("width").getAsInt(), form.get("height").getAsInt());
    }

    @Override
    public Dimension getPreferredSize() {
        Dimension form = formSize();
        return new Dimension(form.width + MARGIN * 2, form.height + MARGIN * 2 + TITLE);
    }

    /** Положение контрола на холсте. */
    private Rectangle frameOf(JsonObject control) {
        JsonArray f = control.getAsJsonArray("frame");
        Point o = origin();
        return new Rectangle(o.x + f.get(0).getAsInt(), o.y + f.get(1).getAsInt(),
                f.get(2).getAsInt(), f.get(3).getAsInt());
    }

    private String hit(Point p) {
        JsonArray controls = controller.controls();
        for (int i = controls.size() - 1; i >= 0; i--) {
            JsonObject control = controls.get(i).getAsJsonObject();
            boolean inside = true;
            // Часть, выступающая за родителя, не видна и не кликается.
            for (JsonObject c = control; c != null && inside; ) {
                inside = frameOf(c).contains(p);
                c = c.get("parent").isJsonNull() ? null : controller.control(c.get("parent").getAsString());
            }
            if (inside) {
                return control.get("name").getAsString();
            }
        }
        return null;
    }

    private static Rectangle knob(Rectangle r, int[] side) {
        int x = side[0] < 0 ? r.x : side[0] > 0 ? r.x + r.width : r.x + r.width / 2;
        int y = side[1] < 0 ? r.y : side[1] > 0 ? r.y + r.height : r.y + r.height / 2;
        return new Rectangle(x - KNOB / 2, y - KNOB / 2, KNOB, KNOB);
    }

    private Rectangle selection() {
        JsonObject control = controller.selected() == null ? null : controller.control(controller.selected());
        return control == null ? null : frameOf(control);
    }

    private Rectangle client() {
        return new Rectangle(origin(), formSize());
    }

    private int[] knobAt(Point p) {
        Rectangle selection = selection();
        // У формы тянутся только правая и нижняя стороны.
        int[][] available = selection != null ? SIDES : new int[][] {{1, 0}, {0, 1}, {1, 1}};
        Rectangle target = selection != null ? selection : client();
        for (int[] side : available) {
            Rectangle k = knob(target, side);
            k.grow(2, 2);
            if (k.contains(p)) {
                return side;
            }
        }
        return null;
    }

    @Override
    protected void paintComponent(Graphics graphics) {
        super.paintComponent(graphics);
        JsonObject form = controller.form();
        Graphics2D g = (Graphics2D) graphics.create();
        double scale = g.getTransform().getScaleX();
        controller.setScale(scale);
        if (form == null) {
            g.drawString("Загрузка формы…", MARGIN, MARGIN);
            g.dispose();
            return;
        }
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR);
        Rectangle client = client();
        g.setColor(new Color(32, 32, 40));
        g.fillRect(client.x, client.y - TITLE, client.width, TITLE);
        g.setColor(Color.WHITE);
        g.drawString(form.get("title").getAsString(), client.x + 10, client.y - TITLE / 2 + 4);
        if (controller.image() != null) {
            g.drawImage(controller.image(), client.x, client.y, client.width, client.height, null);
        }
        for (JsonElement element : controller.controls()) {
            JsonObject control = element.getAsJsonObject();
            Rectangle r = frameOf(control);
            if (!control.get("visible").getAsBoolean()) {
                g.setColor(new Color(255, 255, 255, 170));
                g.fill(r);
            }
            if (!control.get("custom").isJsonNull()) {
                String path = control.get("custom").getAsString();
                g.setColor(ACCENT);
                g.drawString(path.substring(path.lastIndexOf(':') + 1), r.x, r.y - 2);
            }
        }

        Rectangle selection = selection();
        Rectangle target = selection != null ? selection : client;
        g.setColor(ACCENT);
        if (selection != null) {
            g.draw(new Rectangle(selection.x - 1, selection.y - 1, selection.width + 1, selection.height + 1));
        }
        int[][] knobs = selection != null ? SIDES : new int[][] {{1, 0}, {0, 1}, {1, 1}};
        for (int[] side : knobs) {
            Rectangle k = knob(target, side);
            g.setColor(Color.WHITE);
            g.fill(k);
            g.setColor(ACCENT);
            g.draw(k);
        }
        if (ghost != null) {
            g.setStroke(new BasicStroke(1, BasicStroke.CAP_BUTT, BasicStroke.JOIN_MITER, 10, new float[] {4, 3}, 0));
            g.draw(ghost);
        }
        g.dispose();
    }

    private final class Mouse extends MouseAdapter {
        @Override
        public void mousePressed(MouseEvent e) {
            requestFocusInWindow();
            if (e.getButton() != MouseEvent.BUTTON1 || controller.form() == null) {
                return;
            }
            Point p = e.getPoint();
            Point o = origin();
            String tool = controller.tool();
            if (tool != null) {
                if (client().contains(p)) {
                    controller.add(tool, snapped(p.x - o.x, e.getModifiersEx()),
                            snapped(p.y - o.y, e.getModifiersEx()));
                    controller.setTool(null);
                }
                return;
            }
            if (e.getClickCount() == 2) {
                if (client().contains(p)) {
                    controller.handler(hit(p), null);
                }
                return;
            }
            int[] knob = knobAt(p);
            if (knob == null) {
                String name = hit(p);
                controller.select(name);
                if (name == null) {
                    return;
                }
                sides = new int[] {0, 0};
            } else {
                sides = knob;
            }
            pressed = p;
            dragged = controller.selected();
            start = dragged == null ? client() : selection();
        }

        @Override
        public void mouseDragged(MouseEvent e) {
            // Пока нет ответа на прошлую команду, положение на холсте устарело.
            if (pressed == null || controller.busy()) {
                return;
            }
            int dx = e.getX() - pressed.x;
            int dy = e.getY() - pressed.y;
            if (ghost == null && Math.abs(dx) < 4 && Math.abs(dy) < 4) {
                return;
            }
            Point o = origin();
            int mods = e.getModifiersEx();
            // Координаты считаются от угла формы, чтобы сетка совпадала с её сеткой.
            Point parent = parentOrigin();
            int left = start.x - parent.x;
            int top = start.y - parent.y;
            int right = left + start.width;
            int bottom = top + start.height;
            if (dragged != null && sides[0] == 0 && sides[1] == 0) {
                left = snapped(left + dx, mods);
                top = snapped(top + dy, mods);
                right = left + start.width;
                bottom = top + start.height;
            } else {
                if (sides[0] < 0) {
                    left = Math.min(snapped(left + dx, mods), right - KNOB);
                } else if (sides[0] > 0) {
                    right = Math.max(snapped(right + dx, mods), left + KNOB);
                }
                if (sides[1] < 0) {
                    top = Math.min(snapped(top + dy, mods), bottom - KNOB);
                } else if (sides[1] > 0) {
                    bottom = Math.max(snapped(bottom + dy, mods), top + KNOB);
                }
            }
            if (dragged == null) {
                ghost = new Rectangle(o.x, o.y, Math.max(80, right - left), Math.max(40, bottom - top));
            } else {
                ghost = new Rectangle(parent.x + left, parent.y + top, right - left, bottom - top);
            }
            repaint();
        }

        @Override
        public void mouseReleased(MouseEvent e) {
            if (ghost != null) {
                if (dragged == null) {
                    var request = FormController.command("formSize");
                    request.addProperty("width", ghost.width);
                    request.addProperty("height", ghost.height);
                    controller.send(request);
                } else {
                    Point parent = parentOrigin();
                    bounds(dragged, ghost.x - parent.x, ghost.y - parent.y, ghost.width, ghost.height,
                            sides[0] == 0 && sides[1] == 0);
                }
            }
            pressed = null;
            ghost = null;
            repaint();
        }

        @Override
        public void mouseMoved(MouseEvent e) {
            int[] side = controller.form() == null ? null : knobAt(e.getPoint());
            int cursor = Cursor.DEFAULT_CURSOR;
            if (side != null) {
                cursor = side[0] == 0 ? Cursor.N_RESIZE_CURSOR
                        : side[1] == 0 ? Cursor.E_RESIZE_CURSOR
                        : side[0] == side[1] ? Cursor.NW_RESIZE_CURSOR : Cursor.NE_RESIZE_CURSOR;
            } else if (controller.tool() != null) {
                cursor = Cursor.CROSSHAIR_CURSOR;
            }
            setCursor(Cursor.getPredefinedCursor(cursor));
        }

        /** Начало координат родителя перетаскиваемого контрола. */
        private Point parentOrigin() {
            JsonObject control = dragged == null ? null : controller.control(dragged);
            if (control == null || control.get("parent").isJsonNull()) {
                return origin();
            }
            return frameOf(controller.control(control.get("parent").getAsString())).getLocation();
        }
    }

    private void bounds(String name, int x, int y, int width, int height, boolean moved) {
        var request = FormController.command("bounds");
        request.addProperty("name", name);
        request.addProperty("x", x);
        request.addProperty("y", y);
        request.addProperty("width", width);
        request.addProperty("height", height);
        request.addProperty("moved", moved);
        controller.send(request);
    }

    /** Стрелки: сдвиг на пиксель, с Shift — изменение размера. */
    public void nudge(int dx, int dy, boolean resize) {
        JsonObject c = controller.selected() == null ? null : controller.control(controller.selected());
        if (c == null || controller.busy()) {
            return;
        }
        bounds(controller.selected(),
                c.get("x").getAsInt() + (resize ? 0 : dx), c.get("y").getAsInt() + (resize ? 0 : dy),
                c.get("width").getAsInt() + (resize ? dx : 0), c.get("height").getAsInt() + (resize ? dy : 0),
                false);
    }

    /** Escape: убрать инструмент или перейти к родителю. */
    public void cancel() {
        if (controller.tool() != null) {
            controller.setTool(null);
            return;
        }
        JsonObject c = controller.selected() == null ? null : controller.control(controller.selected());
        controller.select(c == null || c.get("parent").isJsonNull() ? null : c.get("parent").getAsString());
    }

    /** Приём типа контрола, перетащенного из Toolbox. */
    private final class Drop extends TransferHandler {
        @Override
        public boolean canImport(TransferSupport support) {
            return support.isDataFlavorSupported(DataFlavor.stringFlavor);
        }

        @Override
        public boolean importData(TransferSupport support) {
            try {
                String data = (String) support.getTransferable().getTransferData(DataFlavor.stringFlavor);
                Point p = support.getDropLocation().getDropPoint();
                if (!data.startsWith(DRAG_PREFIX) || !client().contains(p)) {
                    return false;
                }
                Point o = origin();
                controller.add(data.substring(DRAG_PREFIX.length()), snapped(p.x - o.x, 0), snapped(p.y - o.y, 0));
                controller.setTool(null);
                return true;
            } catch (java.awt.datatransfer.UnsupportedFlavorException | java.io.IOException e) {
                return false;
            }
        }
    }
}
