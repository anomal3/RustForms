package dev.rustforms.rustrover.ui;

import java.awt.BorderLayout;
import java.awt.Dimension;
import java.util.function.Consumer;
import javax.swing.BorderFactory;
import javax.swing.JPanel;
import javax.swing.JToggleButton;

/** Схема Dock, как в WinForms: края, середина (Fill) и None. */
final class DockPicker extends JPanel {
    private final String current;
    private final Consumer<String> choose;

    DockPicker(String current, Consumer<String> choose) {
        super(new BorderLayout(0, 3));
        this.current = current;
        this.choose = choose;
        setBorder(BorderFactory.createEmptyBorder(4, 4, 4, 4));
        JPanel box = new JPanel(new BorderLayout(3, 3));
        box.add(button("", "top", 128, 26), BorderLayout.NORTH);
        box.add(button("", "left", 28, 62), BorderLayout.WEST);
        box.add(button("Fill", "fill", 66, 62), BorderLayout.CENTER);
        box.add(button("", "right", 28, 62), BorderLayout.EAST);
        box.add(button("", "bottom", 128, 26), BorderLayout.SOUTH);
        add(box);
        add(button("None", "none", 128, 26), BorderLayout.SOUTH);
    }

    private JToggleButton button(String text, String side, int width, int height) {
        JToggleButton button = new JToggleButton(text, side.equals(current));
        button.setName(side);
        button.setToolTipText(Character.toUpperCase(side.charAt(0)) + side.substring(1));
        button.setPreferredSize(new Dimension(width, height));
        button.setFocusable(false);
        button.addActionListener(e -> choose.accept(side));
        return button;
    }
}
