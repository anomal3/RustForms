package dev.rustforms.rustrover.ui;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.Rectangle;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import javax.swing.BorderFactory;
import javax.swing.DefaultCellEditor;
import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JComboBox;
import javax.swing.JComponent;
import javax.swing.JPanel;
import javax.swing.JPopupMenu;
import javax.swing.JScrollPane;
import javax.swing.JSplitPane;
import javax.swing.JTable;
import javax.swing.JTextField;
import javax.swing.table.AbstractTableModel;
import javax.swing.table.TableCellEditor;
import javax.swing.table.TableCellRenderer;

/** Окно свойств и событий, как Properties в Visual Studio. */
public final class PropertiesPanel extends JPanel {
    private enum Type { TEXT, NUMBER, FLAG, EVENT, DOCK, ANCHOR, ALIGN }

    private record Row(String label, String owner, String property, Type type, Object value) {
    }

    private final FormController controller;
    private final List<Row> rows = new ArrayList<>();
    private final List<Row> events = new ArrayList<>();
    private final JComboBox<String> chooser = new JComboBox<>();
    private final JButton go = new JButton("Перейти к обработчику");
    private final Model model = new Model(rows, "Свойство");
    private final Model eventModel = new Model(events, "Событие");
    private boolean refreshing;

    private final JTable table = new JTable(model) {
        @Override
        public TableCellRenderer getCellRenderer(int row, int column) {
            return column == 1 && rows.get(row).type() == Type.FLAG
                    ? getDefaultRenderer(Boolean.class) : super.getCellRenderer(row, column);
        }

        @Override
        public TableCellEditor getCellEditor(int row, int column) {
            return switch (rows.get(row).type()) {
                case FLAG -> new DefaultCellEditor(flagBox());
                case ALIGN -> new DefaultCellEditor(new JComboBox<>(
                        new String[] {"", "left", "center", "right"}));
                default -> new DefaultCellEditor(new JTextField());
            };
        }

        @Override
        public Component prepareEditor(TableCellEditor editor, int row, int column) {
            Component component = super.prepareEditor(editor, row, column);
            if (component instanceof JCheckBox box) {
                box.setSelected(Boolean.TRUE.equals(getValueAt(row, column)));
            }
            return component;
        }
    };

    private final JTable eventTable = new JTable(eventModel);

    private static JCheckBox flagBox() {
        JCheckBox box = new JCheckBox();
        box.setHorizontalAlignment(JCheckBox.CENTER);
        return box;
    }

    public PropertiesPanel(FormController controller) {
        super(new BorderLayout(0, 4));
        this.controller = controller;
        for (JTable each : new JTable[] {table, eventTable}) {
            each.setRowHeight(24);
            each.getTableHeader().setReorderingAllowed(false);
            each.putClientProperty("terminateEditOnFocusLost", Boolean.TRUE);
        }
        // Свойства сверху, события снизу; границу можно двигать мышью.
        JPanel lower = new JPanel(new BorderLayout(0, 4));
        lower.add(new JScrollPane(eventTable));
        lower.add(go, BorderLayout.SOUTH);
        JSplitPane split = new JSplitPane(JSplitPane.VERTICAL_SPLIT, new JScrollPane(table), lower);
        split.setResizeWeight(0.72);
        split.setContinuousLayout(true);
        split.setBorder(BorderFactory.createEmptyBorder());
        add(chooser, BorderLayout.NORTH);
        add(split);

        chooser.addActionListener(e -> {
            if (!refreshing) {
                int index = chooser.getSelectedIndex();
                controller.select(index <= 0 ? null : (String) chooser.getSelectedItem());
            }
        });
        go.addActionListener(e -> goToHandler());
        eventTable.getSelectionModel().addListSelectionListener(e -> updateButton());
        eventTable.addMouseListener(new MouseAdapter() {
            @Override
            public void mousePressed(MouseEvent e) {
                if (e.getClickCount() == 2 && eventTable.columnAtPoint(e.getPoint()) == 0) {
                    goToHandler();
                }
            }
        });
        // Dock и Anchor выбираются картинкой, как в WinForms, а не вводом в ячейку.
        table.addMouseListener(new MouseAdapter() {
            @Override
            public void mousePressed(MouseEvent e) {
                int row = table.rowAtPoint(e.getPoint());
                if (row >= 0 && table.columnAtPoint(e.getPoint()) == 1) {
                    showPicker(row);
                }
            }
        });
        controller.listen(this::refresh);
        refresh();
    }

    private void showPicker(int index) {
        Row row = rows.get(index);
        if (row.type() != Type.DOCK && row.type() != Type.ANCHOR) {
            return;
        }
        if (table.isEditing()) {
            table.getCellEditor().stopCellEditing();
        }
        JPopupMenu popup = new JPopupMenu();
        popup.add(picker(row.owner(), row.type(), popup));
        Rectangle cell = table.getCellRect(index, 1, true);
        popup.show(table, cell.x, cell.y + cell.height);
    }

    private JComponent picker(String owner, Type type, JPopupMenu popup) {
        JsonObject control = controller.control(owner);
        if (type == Type.DOCK) {
            return new DockPicker(text(control, "dock"), side -> {
                popup.setVisible(false);
                controller.set(owner, "dock", side);
            });
        }
        Set<String> sides = new HashSet<>();
        control.getAsJsonArray("anchor").forEach(side -> sides.add(side.getAsString()));
        return new AnchorPicker(sides, (side, on) -> {
            // Как в WinForms: якоря и Dock исключают друг друга.
            if (!text(controller.control(owner), "dock").equals("none")) {
                controller.set(owner, "dock", "none");
            }
            controller.set(owner, "anchor:" + side, on);
        });
    }

    /** Окошко выбора Dock или Anchor. Видно и проверке вне IDE. */
    public JComponent picker(String label) {
        for (Row row : rows) {
            if (row.label().equals(label)) {
                return picker(row.owner(), row.type(), new JPopupMenu());
            }
        }
        throw new IllegalArgumentException("нет строки " + label);
    }

    private static String capital(String word) {
        return word.isEmpty() ? word : Character.toUpperCase(word.charAt(0)) + word.substring(1);
    }

    private Row eventRow() {
        int row = eventTable.getSelectedRow();
        return row >= 0 && row < events.size() ? events.get(row) : null;
    }

    private void updateButton() {
        go.setEnabled(eventRow() != null);
    }

    private void goToHandler() {
        Row row = eventRow();
        if (row != null) {
            controller.handler(row.owner(), row.property());
        }
    }

    private static String text(JsonObject object, String key) {
        JsonElement value = object.get(key);
        return value == null || value.isJsonNull() ? "" : value.getAsString();
    }

    private void refresh() {
        JsonObject form = controller.form();
        String selected = controller.selected();
        JsonObject control = selected == null ? null : controller.control(selected);
        List<Row> next = new ArrayList<>();
        if (control != null) {
            String kind = text(control, "kind");
            next.add(new Row("Имя", selected, "name", Type.TEXT, selected));
            if (List.of("Button", "Label", "TextBox", "CheckBox", "RadioButton", "GroupBox", "LinkLabel", "StatusStrip", "TabPage")
                    .contains(kind)) {
                // Переводы строк многострочного поля показываются как \n.
                String value = text(control, "text");
                if (control.get("multiline").getAsBoolean()) {
                    value = value.replace("\n", "\\n");
                }
                next.add(new Row("Текст", selected, "text", Type.TEXT, value));
            }
            for (String[] p : new String[][] {{"X", "x"}, {"Y", "y"}, {"Ширина", "width"}, {"Высота", "height"}}) {
                next.add(new Row(p[0], selected, p[1], Type.NUMBER, control.get(p[1]).getAsInt()));
            }
            // У таймера нет внешнего вида: только «запущен», интервал и событие.
            int appearance = next.size();
            next.add(new Row("Цвет текста (#rrggbb)", selected, "fore_color", Type.TEXT, text(control, "fore_color")));
            next.add(new Row("Фон (#rrggbb)", selected, "back_color", Type.TEXT, text(control, "back_color")));
            JsonElement size = control.get("font_size");
            next.add(new Row("Шрифт (0 — обычный)", selected, "font_size", Type.NUMBER,
                    size.isJsonNull() ? 0 : size.getAsInt()));
            next.add(new Row("Жирный", selected, "bold", Type.FLAG, control.get("bold").getAsBoolean()));
            next.add(new Row("Подсказка", selected, "tooltip", Type.TEXT, text(control, "tooltip")));
            next.add(new Row("Порядок Tab", selected, "tab_index", Type.NUMBER, control.get("tab_index").getAsInt()));
            if (List.of("Button", "Label", "LinkLabel", "StatusStrip").contains(kind)) {
                next.add(new Row("Выравнивание", selected, "align", Type.ALIGN, text(control, "align")));
            }
            if (kind.equals("TextBox")) {
                next.add(new Row("Только чтение", selected, "read_only", Type.FLAG, control.get("read_only").getAsBoolean()));
                next.add(new Row("Пароль", selected, "password", Type.FLAG, control.get("password").getAsBoolean()));
            }
            next.add(new Row("Dock", selected, "dock", Type.DOCK, capital(text(control, "dock"))));
            List<String> sides = new ArrayList<>();
            for (String side : new String[] {"top", "bottom", "left", "right"}) {
                if (control.getAsJsonArray("anchor").contains(new JsonPrimitive(side))) {
                    sides.add(capital(side));
                }
            }
            next.add(new Row("Anchor", selected, "anchor", Type.ANCHOR,
                    sides.isEmpty() ? "None" : String.join(", ", sides)));
            next.add(new Row("Видимый", selected, "visible", Type.FLAG, control.get("visible").getAsBoolean()));
            next.add(new Row("Доступен", selected, "enabled", Type.FLAG, control.get("enabled").getAsBoolean()));
            if (kind.equals("TextBox")) {
                next.add(new Row("Многострочный", selected, "multiline", Type.FLAG, control.get("multiline").getAsBoolean()));
            }
            if (List.of("ComboBox", "ListBox", "MenuStrip", "DataGridView", "TreeView", "ToolStrip").contains(kind)) {
                List<String> items = new ArrayList<>();
                control.getAsJsonArray("items").forEach(item -> items.add(item.getAsString()));
                next.add(new Row("Строки (через ;)", selected, "items", Type.TEXT, String.join("; ", items)));
                if (kind.equals("ComboBox") || kind.equals("ListBox")) {
                    JsonElement chosen = control.get("selected");
                    next.add(new Row("Выбрана (с 0, -1 нет)", selected, "selected", Type.NUMBER,
                            chosen.isJsonNull() ? -1 : chosen.getAsInt()));
                }
            }
            if (kind.equals("TabControl")) {
                JsonElement page = control.get("selected");
                next.add(new Row("Страница (с 0)", selected, "selected", Type.NUMBER,
                        page.isJsonNull() ? 0 : page.getAsInt()));
            }
            if (kind.equals("Timer")) {
                next.add(new Row("Интервал, мс", selected, "interval", Type.NUMBER, control.get("interval").getAsInt()));
            }
            if (kind.equals("DateTimePicker")) {
                next.add(new Row("Дата (ГГГГ-ММ-ДД)", selected, "date", Type.TEXT, text(control, "date")));
            }
            if (kind.equals("SplitContainer")) {
                next.add(new Row("Разделитель", selected, "distance", Type.NUMBER, control.get("distance").getAsInt()));
                next.add(new Row("Горизонтальный", selected, "horizontal", Type.FLAG, control.get("horizontal").getAsBoolean()));
            }
            if (kind.equals("DataGridView")) {
                next.add(new Row("Правка ячеек", selected, "editable", Type.FLAG, control.get("editable").getAsBoolean()));
            }
            if (!kind.equals("Timer")) {
                List<String> menu = new ArrayList<>();
                control.getAsJsonArray("context").forEach(item -> menu.add(item.getAsString()));
                next.add(new Row("Меню правой кнопки (через ;)", selected, "context", Type.TEXT, String.join("; ", menu)));
            }
            if (kind.equals("PictureBox")) {
                next.add(new Row("Файл PNG (от папки формы)", selected, "image", Type.TEXT, text(control, "image")));
            }
            if (List.of("ProgressBar", "NumericUpDown", "TrackBar").contains(kind)) {
                for (String[] p : new String[][] {{"Минимум", "minimum"}, {"Максимум", "maximum"}, {"Значение", "value"}}) {
                    next.add(new Row(p[0], selected, p[1], Type.NUMBER, control.get(p[1]).getAsInt()));
                }
            }
            if (kind.equals("CheckBox") || kind.equals("RadioButton")) {
                next.add(new Row("Отмечен", selected, "checked", Type.FLAG, control.get("checked").getAsBoolean()));
            }
            if (kind.equals("Panel")) {
                next.add(new Row("Рамка", selected, "border", Type.FLAG, control.get("border").getAsBoolean()));
            }
            if (kind.equals("Timer")) {
                next.subList(appearance, next.size()).removeIf(r -> !r.property().equals("enabled"));
                next.removeIf(r -> List.of("x", "y", "width", "height", "text").contains(r.property()));
            }
            next.add(new Row("Свой тип", selected, "custom", Type.TEXT, text(control, "custom")));
            for (JsonElement element : control.getAsJsonArray("events")) {
                JsonObject event = element.getAsJsonObject();
                String name = text(event, "event");
                next.add(new Row("⚡ " + name, selected, name, Type.EVENT, text(event, "handler")));
            }
        } else if (form != null) {
            next.add(new Row("Структура", null, "name", Type.TEXT, text(form, "name")));
            next.add(new Row("Заголовок", null, "title", Type.TEXT, text(form, "title")));
            next.add(new Row("Ширина", null, "width", Type.NUMBER, form.get("width").getAsInt()));
            next.add(new Row("Высота", null, "height", Type.NUMBER, form.get("height").getAsInt()));
            next.add(new Row("Фон (#rrggbb)", null, "back_color", Type.TEXT, text(form, "back_color")));
            for (String event : new String[] {"load", "closing", "resize"}) {
                next.add(new Row("⚡ " + event, null, event, Type.EVENT, text(form, event)));
            }
        }

        refreshing = true;
        chooser.removeAllItems();
        if (form != null) {
            chooser.addItem(text(form, "name") + " (форма)");
            for (JsonElement element : controller.controls()) {
                chooser.addItem(text(element.getAsJsonObject(), "name"));
            }
            if (control != null) {
                chooser.setSelectedItem(selected);
            }
        }
        refreshing = false;

        fill(table, model, rows, next.stream().filter(r -> r.type() != Type.EVENT).toList());
        fill(eventTable, eventModel, events, next.stream().filter(r -> r.type() == Type.EVENT).toList());
        updateButton();
    }

    private static void fill(JTable table, Model model, List<Row> target, List<Row> next) {
        if (next.equals(target)) {
            return;
        }
        if (table.isEditing()) {
            table.getCellEditor().cancelCellEditing();
        }
        int keep = table.getSelectedRow();
        target.clear();
        target.addAll(next);
        model.fireTableDataChanged();
        if (keep >= 0 && keep < target.size()) {
            table.setRowSelectionInterval(keep, keep);
        }
    }

    /** Применить введённое значение. Видно и проверке вне IDE. */
    public void edit(String label, Object value) {
        for (Model each : new Model[] {model, eventModel}) {
            for (int i = 0; i < each.rows.size(); i++) {
                if (each.rows.get(i).label().equals(label)) {
                    each.setValueAt(value, i, 1);
                    return;
                }
            }
        }
        throw new IllegalArgumentException("нет строки " + label);
    }

    private final class Model extends AbstractTableModel {
        private final List<Row> rows;
        private final String header;

        Model(List<Row> rows, String header) {
            this.rows = rows;
            this.header = header;
        }

        @Override
        public int getRowCount() {
            return rows.size();
        }

        @Override
        public int getColumnCount() {
            return 2;
        }

        @Override
        public String getColumnName(int column) {
            return column == 0 ? header : header.equals("Событие") ? "Обработчик" : "Значение";
        }

        @Override
        public Object getValueAt(int row, int column) {
            return column == 0 ? rows.get(row).label() : rows.get(row).value();
        }

        @Override
        public boolean isCellEditable(int row, int column) {
            return column == 1 && rows.get(row).type() != Type.DOCK && rows.get(row).type() != Type.ANCHOR;
        }

        @Override
        public void setValueAt(Object value, int index, int column) {
            if (index >= rows.size()) {
                return;
            }
            Row row = rows.get(index);
            if (String.valueOf(value).equals(String.valueOf(row.value()))) {
                return;
            }
            Object parsed = value;
            if (row.property().equals("text") && row.owner() != null
                    && controller.control(row.owner()).get("multiline").getAsBoolean()) {
                parsed = String.valueOf(value).replace("\\n", "\n");
            }
            if (row.type() == Type.NUMBER) {
                try {
                    parsed = Double.parseDouble(String.valueOf(value).trim().replace(',', '.'));
                } catch (NumberFormatException e) {
                    return;
                }
            }
            String property = row.type() == Type.EVENT && row.owner() != null
                    ? "event:" + row.property() : row.property();
            controller.set(row.owner(), property, parsed);
        }
    }
}
