package dev.makepad.android;

/**
 * KumkajSlipLayout's arithmetic, run on a desktop JVM — no device, no
 * Android classes. The cases and the numbers are the desktop engine's own
 * tests (apps/khaynjo/src/os_paper.rs), so Android lays a slip out exactly
 * as Windows and Linux do.
 *
 *   javac -d /tmp/slip-layout patches/makepad/KumkajSlipLayout.java \
 *         patches/makepad/KumkajSlipLayoutTest.java
 *   java -cp /tmp/slip-layout dev.makepad.android.KumkajSlipLayoutTest
 *
 * Not part of the app: scripts/apply-android-activity-bridge.sh installs
 * only KumkajSlipLayout.java.
 */
final class KumkajSlipLayoutTest {
    private static int failed = 0;

    private static void same(String what, long got, long want) {
        if (got != want) {
            failed++;
            System.out.println("FAIL " + what + ": got " + got + ", want " + want);
        }
    }

    private static KumkajSlipLayout at(int down, int across, int slipW, int len, boolean roll, boolean onePageOnly) {
        // A 100 × 200 area with no virtual paper.
        return KumkajSlipLayout.plan(slipW, len, new int[0], 100, onePageOnly ? 200 : 200, 0, 0, roll, down * 3 + across);
    }

    public static void main(String[] args) {
        // os_paper::tests::the_nine_places_are_the_papers_own_sides
        same("top left x", at(0, 0, 40, 50, false, true).x, 0);
        same("top left y", at(0, 0, 40, 50, false, true).y, 0);
        same("top centre x", at(0, 1, 40, 50, false, true).x, 30);
        same("top right x", at(0, 2, 40, 50, false, true).x, 60);
        same("middle left y", at(1, 0, 40, 50, false, true).y, 75);
        same("bottom right x", at(2, 2, 40, 50, false, true).x, 60);
        same("bottom right y", at(2, 2, 40, 50, false, true).y, 150);
        // Down only on one sheet with a length that holds the slip.
        same("roll: down does not count", at(2, 0, 40, 50, true, true).y, 0);
        same("longer than the area: at the top", at(2, 0, 40, 250, false, true).y, 0);
        // Wider than the area: it hangs over by the same rule.
        same("wide, left", at(0, 0, 150, 50, false, true).x, 0);
        same("wide, centre", at(0, 1, 150, 50, false, true).x, -25);
        same("wide, right", at(0, 2, 150, 50, false, true).x, -50);

        // os_paper::tests::a_place_is_measured_on_the_virtual_paper: a
        // half-size sheet (95 × 138 mm reachable at 10 px/mm) in a printer
        // whose page is bigger (200 × 287 mm reachable).
        KumkajSlipLayout half = KumkajSlipLayout.plan(400, 20, new int[0], 2000, 2870, 950, 1380, false, 1 * 3 + 1);
        same("half sheet: area width", half.areaW, 950);
        same("half sheet: area height", half.areaH, 1380);
        same("half sheet: centre", half.x, 275);
        same("half sheet: middle", half.y, 680);
        same("half sheet: one page", half.pages.length, 1);

        // A printer whose page is smaller than the virtual paper reaches
        // only its own page.
        KumkajSlipLayout small = KumkajSlipLayout.plan(800, 20, new int[0], 600, 480, 990, 480, false, 0);
        same("small page: area width", small.areaW, 600);

        // os_paper::tests::page_breaks_match_the_android_layout: 30 lines of
        // 20 px, 10 px apart, on pages that hold 250 px.
        int[] gaps = new int[29];
        for (int i = 0; i < gaps.length; i++) {
            gaps[i] = i * 30 + 25;
        }
        KumkajSlipLayout longSlip = KumkajSlipLayout.plan(100, 890, gaps, 100, 250, 0, 0, false, 2 * 3 + 1);
        int[][] want = {{0, 235}, {235, 475}, {475, 715}, {715, 890}};
        same("long slip: pages", longSlip.pages.length, want.length);
        for (int i = 0; i < Math.min(want.length, longSlip.pages.length); i++) {
            same("long slip: page " + i + " start", longSlip.pages[i][0], want[i][0]);
            same("long slip: page " + i + " end", longSlip.pages[i][1], want[i][1]);
        }
        same("long slip: more than one page starts at the top", longSlip.y, 0);

        // A roll longer than the dialog's page goes on to its next page.
        KumkajSlipLayout roll = KumkajSlipLayout.plan(100, 890, gaps, 120, 250, 100, 0, true, 0);
        same("roll: area width is the roll's", roll.areaW, 100);
        same("roll: pages", roll.pages.length, 4);

        // A picture taller than half a page with no blank band: cut at the page.
        KumkajSlipLayout solid = KumkajSlipLayout.plan(50, 900, new int[0], 50, 250, 0, 0, false, 0);
        same("solid: pages", solid.pages.length, 4);
        same("solid: first end", solid.pages[0][1], 250);
        same("solid: last end", solid.pages[3][1], 900);

        if (failed > 0) {
            System.out.println(failed + " failed");
            System.exit(1);
        }
        System.out.println("KumkajSlipLayout: all checks passed");
    }
}
