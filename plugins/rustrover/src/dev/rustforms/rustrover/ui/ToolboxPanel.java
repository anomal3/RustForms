package dev.rustforms.rustrover.ui;

import java.awt.BorderLayout;
import java.awt.datatransfer.StringSelection;
import java.awt.datatransfer.Transferable;
import javax.swing.JComponent;
import javax.swing.JList;
import javax.swing.JPanel;
import javax.swing.JScrollPane;
import javax.swing.ListSelectionModel;
import javax.swing.TransferHandler;

/** Список элементов: щелчок и затем щелчок по форме, либо перетаскивание на неё. */
public final class ToolboxPanel extends JPanel {
    private final JList<String> list = new JList<>(new String[] {"Button", "Label", "TextBox", "CheckBox", "Panel"});

    public ToolboxPanel(FormController controller) {
        super(new BorderLayout());
        list.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
        list.setFixedCellHeight(26);
        list.setDragEnabled(true);
        list.setTransferHandler(new TransferHandler() {
            @Override
            public int getSourceActions(JComponent c) {
                return COPY;
            }

            @Override
            protected Transferable createTransferable(JComponent c) {
                String kind = list.getSelectedValue();
                return kind == null ? null : new StringSelection(FormCanvas.DRAG_PREFIX + kind);
            }
        });
        list.addListSelectionListener(e -> {
            if (!e.getValueIsAdjusting() && list.getSelectedValue() != null) {
                controller.setTool(list.getSelectedValue());
            }
        });
        controller.listen(() -> {
            if (controller.tool() == null) {
                list.clearSelection();
            }
        });
        add(new JScrollPane(list));
    }
}
