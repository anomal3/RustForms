package dev.rustforms.rustrover.ui;

import java.awt.Color;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Rectangle;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.util.HashSet;
import java.util.Set;
import java.util.function.BiConsumer;
import javax.swing.JComponent;
import javax.swing.UIManager;

/** Квадрат Anchor, как в WinForms: щелчок по перекладине включает привязку к краю. */
final class AnchorPicker extends JComponent {
    private static final String[] SIDES = {"left", "top", "right", "bottom"};
    private static final Rectangle[] BARS = {
            new Rectangle(7, 62, 39, 9), new Rectangle(62, 7, 9, 39),
            new Rectangle(87, 62, 39, 9), new Rectangle(62, 87, 9, 39)};
    private static final Rectangle[] HITS = {
            new Rectangle(0, 46, 46, 41), new Rectangle(46, 0, 41, 46),
            new Rectangle(87, 46, 46, 41), new Rectangle(46, 87, 41, 46)};

    private final Set<String> on;

    AnchorPicker(Set<String> current, BiConsumer<String, Boolean> change) {
        on = new HashSet<>(current);
        setPreferredSize(new Dimension(133, 133));
        setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        setToolTipText("Щелчок по перекладине включает или выключает привязку");
        addMouseListener(new MouseAdapter() {
            @Override
            public void mousePressed(MouseEvent e) {
                for (int i = 0; i < SIDES.length; i++) {
                    if (HITS[i].contains(e.getPoint())) {
                        boolean value = !on.remove(SIDES[i]);
                        if (value) {
                            on.add(SIDES[i]);
                        }
                        change.accept(SIDES[i], value);
                        repaint();
                    }
                }
            }
        });
    }

    @Override
    protected void paintComponent(Graphics g) {
        Color back = UIManager.getColor("Table.background");
        Color line = UIManager.getColor("Label.foreground");
        g.setColor(back == null ? Color.WHITE : back);
        g.fillRect(0, 0, getWidth(), getHeight());
        g.setColor(Color.GRAY);
        g.drawRect(6, 6, 120, 120);
        g.fillRect(46, 46, 41, 41);
        for (int i = 0; i < SIDES.length; i++) {
            Rectangle bar = BARS[i];
            if (on.contains(SIDES[i])) {
                g.setColor(new Color(0x3574F0));
                g.fillRect(bar.x, bar.y, bar.width, bar.height);
            } else {
                g.setColor(line == null ? Color.DARK_GRAY : line);
                g.drawRect(bar.x, bar.y, bar.width - 1, bar.height - 1);
            }
        }
    }
}
