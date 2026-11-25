package dev.rustforms.rustrover.ui;

import java.awt.Component;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.InputStream;
import javax.imageio.ImageIO;
import javax.swing.Icon;

/** Значок 16×16 из картинки вдвое крупнее: на экранах с масштабом остаётся чётким. */
final class ToolIcon implements Icon {
    private final BufferedImage image;

    private ToolIcon(BufferedImage image) {
        this.image = image;
    }

    static Icon load(String kind) {
        try (InputStream in = ToolIcon.class.getResourceAsStream("/icons/" + kind + ".png")) {
            return in == null ? null : new ToolIcon(ImageIO.read(in));
        } catch (IOException e) {
            return null;
        }
    }

    @Override
    public void paintIcon(Component c, Graphics g, int x, int y) {
        Graphics2D copy = (Graphics2D) g.create();
        copy.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BICUBIC);
        copy.drawImage(image, x, y, getIconWidth(), getIconHeight(), null);
        copy.dispose();
    }

    @Override
    public int getIconWidth() {
        return 16;
    }

    @Override
    public int getIconHeight() {
        return 16;
    }
}
