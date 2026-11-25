package dev.rustforms.rustrover.ui;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.util.ArrayList;
import java.util.List;
import javax.swing.DefaultCellEditor;
import javax.swing.JButton;
import javax.swing.JCheckBox;
import javax.swing.JComboBox;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.JTable;
import javax.swing.JTextField;
import javax.swing.table.AbstractTableModel;
import javax.swing.table.TableCellEditor;
import javax.swing.table.TableCellRenderer;

/** Окно свойств и событий, как Properties в Visual Studio. */
public final class PropertiesPanel extends JPanel {
    private enum Type { TEXT, NUMBER, FLAG, EVENT }

    private record Row(String label, String owner, String property, Type type, Object value) {
    }

    private final FormController controller;
    private final List<Row> rows = new ArrayList<>();
    private final JComboBox<String> chooser = new JComboBox<>();
    private final JButton go = new JButton("Перейти к обработчику");
    private final Model model = new Model();
    private boolean refreshing;

    private final JTable table = new JTable(model) {
        @Override
        public TableCellRenderer getCellRenderer(int row, int column) {
            return column == 1 && rows.get(row).type() == Type.FLAG
                    ? getDefaultRenderer(Boolean.class) : super.getCellRenderer(row, column);
        }

        @Override
        public TableCellEditor getCellEditor(int row, int column) {
            return rows.get(row).type() == Type.FLAG
                    ? new DefaultCellEditor(flagBox()) : new DefaultCellEditor(new JTextField());
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

    private static JCheckBox flagBox() {
        JCheckBox box = new JCheckBox();
        box.setHorizontalAlignment(JCheckBox.CENTER);
        return box;
    }

    public PropertiesPanel(FormController controller) {
        super(new BorderLayout(0, 4));
        this.controller = controller;
        table.setRowHeight(24);
        table.getTableHeader().setReorderingAllowed(false);
        table.putClientProperty("terminateEditOnFocusLost", Boolean.TRUE);
        add(chooser, BorderLayout.NORTH);
        add(new JScrollPane(table));
        add(go, BorderLayout.SOUTH);

        chooser.addActionListener(e -> {
            if (!refreshing) {
                int index = chooser.getSelectedIndex();
                controller.select(index <= 0 ? null : (String) chooser.getSelectedItem());
            }
        });
        go.addActionListener(e -> goToHandler());
        table.getSelectionModel().addListSelectionListener(e -> updateButton());
        table.addMouseListener(new MouseAdapter() {
            @Override
            public void mousePressed(MouseEvent e) {
                if (e.getClickCount() == 2 && table.columnAtPoint(e.getPoint()) == 0) {
                    goToHandler();
                }
            }
        });
        controller.listen(this::refresh);
        refresh();
    }

    private Row eventRow() {
        int row = table.getSelectedRow();
        return row >= 0 && row < rows.size() && rows.get(row).type() == Type.EVENT ? rows.get(row) : null;
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
            if (!kind.equals("Panel")) {
                next.add(new Row("Текст", selected, "text", Type.TEXT, text(control, "text")));
            }
            for (String[] p : new String[][] {{"X", "x"}, {"Y", "y"}, {"Ширина", "width"}, {"Высота", "height"}}) {
                next.add(new Row(p[0], selected, p[1], Type.NUMBER, control.get(p[1]).getAsInt()));
            }
            next.add(new Row("Видимый", selected, "visible", Type.FLAG, control.get("visible").getAsBoolean()));
            next.add(new Row("Доступен", selected, "enabled", Type.FLAG, control.get("enabled").getAsBoolean()));
            if (kind.equals("CheckBox")) {
                next.add(new Row("Отмечен", selected, "checked", Type.FLAG, control.get("checked").getAsBoolean()));
            }
            if (kind.equals("Label")) {
                next.add(new Row("Цвет текста (#rrggbb)", selected, "fore_color", Type.TEXT, text(control, "fore_color")));
            }
            if (kind.equals("Panel")) {
                next.add(new Row("Фон (#rrggbb)", selected, "back_color", Type.TEXT, text(control, "back_color")));
                next.add(new Row("Рамка", selected, "border", Type.FLAG, control.get("border").getAsBoolean()));
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
            next.add(new Row("⚡ load", null, "load", Type.EVENT, text(form, "load")));
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

        if (!next.equals(rows)) {
            if (table.isEditing()) {
                table.getCellEditor().cancelCellEditing();
            }
            int keep = table.getSelectedRow();
            rows.clear();
            rows.addAll(next);
            model.fireTableDataChanged();
            if (keep >= 0 && keep < rows.size()) {
                table.setRowSelectionInterval(keep, keep);
            }
        }
        updateButton();
    }

    /** Применить введённое значение. Видно и проверке вне IDE. */
    public void edit(String label, Object value) {
        for (int i = 0; i < rows.size(); i++) {
            if (rows.get(i).label().equals(label)) {
                model.setValueAt(value, i, 1);
                return;
            }
        }
        throw new IllegalArgumentException("нет строки " + label);
    }

    private final class Model extends AbstractTableModel {
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
            return column == 0 ? "Свойство" : "Значение";
        }

        @Override
        public Object getValueAt(int row, int column) {
            return column == 0 ? rows.get(row).label() : rows.get(row).value();
        }

        @Override
        public boolean isCellEditable(int row, int column) {
            return column == 1;
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
