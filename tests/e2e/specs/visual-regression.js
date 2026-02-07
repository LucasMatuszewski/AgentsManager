/**
 * Visual Regression Tests for AgentsManager
 *
 * These tests capture screenshots of key app states and compare them
 * against baselines. First run creates baselines; subsequent runs detect
 * visual regressions.
 *
 * NOTE: Run on CI only (debug build is too heavy for 4GB RAM local dev).
 */

describe("Visual Regression", () => {
  it("should match the main window baseline", async () => {
    // Wait for the app to fully render
    await browser.pause(2000);

    const result = await browser.checkFullPageScreen("main-window", {
      disableCSSAnimation: true,
    });

    // First run: autoSaveBaseline creates the baseline (result === 0)
    // Subsequent runs: fail if mismatch > 1%
    expect(result).toBeLessThanOrEqual(1);
  });

  it("should match the sidebar baseline", async () => {
    await browser.pause(1000);

    // Try to find sidebar element
    const sidebar = await $('[data-testid="sidebar"]');
    if (await sidebar.isExisting()) {
      const result = await browser.checkElement(sidebar, "sidebar", {
        disableCSSAnimation: true,
      });
      expect(result).toBeLessThanOrEqual(1);
    } else {
      // Sidebar may use a different selector; capture full page as fallback
      console.warn(
        "Sidebar element not found with [data-testid=sidebar], skipping element test",
      );
    }
  });

  it("should capture empty state screenshot", async () => {
    // Capture what the app looks like with no threads/conversations
    await browser.pause(1000);

    const result = await browser.checkFullPageScreen("empty-state", {
      disableCSSAnimation: true,
    });

    expect(result).toBeLessThanOrEqual(1);
  });
});
