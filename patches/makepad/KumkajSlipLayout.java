package dev.makepad.android;

/**
 * Where an OS-document slip's pages go on the paper picked in the system
 * print dialog (MakepadActivity.printSlipPagesPlaced). Plain arithmetic, no
 * Android classes, so it also runs on a desktop JVM
 * (patches/makepad/KumkajSlipLayoutTest.java).
 *
 * The same rule as the desktop apps' os_paper.rs (sheet_for, plan_sheets,
 * place_in, page_ends): a page is laid on the part of the dialog's paper
 * that the printer reaches and the virtual paper covers; the slip sits there
 * at its place — left, centre or right, and, when the whole slip is on one
 * page of a sheet, top, middle or bottom; a longer slip goes on to more
 * pages, split in the blank band between two lines.
 */
final class KumkajSlipLayout {
    /** The area a page is laid on, in slip pixels. */
    final int areaW;
    final int areaH;
    /** Rows {start, end} of the slip on each page; the end is not printed. */
    final int[][] pages;
    /**
     * The slip's left edge and top on a page, in slip pixels from the area's
     * corner. A negative left: the slip starts past the left edge.
     */
    final int x;
    final int y;

    private KumkajSlipLayout(int areaW, int areaH, int[][] pages, int x, int y) {
        this.areaW = areaW;
        this.areaH = areaH;
        this.pages = pages;
        this.x = x;
        this.y = y;
    }

    /**
     * @param slipW    the slip's width, pixels
     * @param len      its inked length, pixels
     * @param gaps     rows where a page may end (the middle of each blank
     *                 band between lines), ascending
     * @param pageW    the dialog paper's printable width, in slip pixels
     * @param pageH    its printable height, in slip pixels
     * @param virtualW the virtual paper's reachable width, in slip pixels;
     *                 0 = no limit
     * @param virtualH its reachable height; 0 = no limit (none set, or a
     *                 roll)
     * @param roll     the virtual paper is a roll: down does not count
     * @param place    down × 3 + across: 0 top / left … 2 bottom / right
     */
    static KumkajSlipLayout plan(
            int slipW, int len, int[] gaps,
            double pageW, double pageH, double virtualW, double virtualH,
            boolean roll, int place) {
        // Floor: a page never reaches past the area's edge.
        int areaW = Math.max(1, (int) Math.floor(virtualW > 0 ? Math.min(pageW, virtualW) : pageW));
        int areaH = Math.max(1, (int) Math.floor(virtualH > 0 ? Math.min(pageH, virtualH) : pageH));
        len = Math.max(1, len);
        java.util.ArrayList<int[]> pages = new java.util.ArrayList<>();
        int start = 0;
        while (start < len) {
            long limit = (long) start + areaH;
            if (limit >= len) {
                pages.add(new int[]{start, len});
                break;
            }
            // The last blank band in the page's second half, so no line is
            // cut in two; the page's own end when it has none.
            int floor = start + areaH / 2;
            int end = (int) limit;
            for (int i = gaps.length - 1; i >= 0; i--) {
                if (gaps[i] > floor && gaps[i] <= limit) {
                    end = gaps[i];
                    break;
                }
            }
            pages.add(new int[]{start, end});
            start = end;
        }
        int across = Math.max(0, Math.min(2, place % 3));
        int down = Math.max(0, Math.min(2, place / 3));
        // Whole-number division, toward zero: the desktop's own.
        int x = (areaW - slipW) * across / 2;
        int y = (!roll && pages.size() == 1 && len <= areaH) ? (areaH - len) * down / 2 : 0;
        return new KumkajSlipLayout(areaW, areaH, pages.toArray(new int[0][]), x, y);
    }
}
