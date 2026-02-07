/**
 * Basic E2E test: verify the AgentsManager app launches and renders the main UI.
 *
 * Requires:
 *  - A debug build: `pnpm tauri build --debug --no-bundle`
 *  - tauri-driver installed: `cargo install tauri-driver --locked`
 *  - WebKitWebDriver available (webkitgtk-6.0 on Arch, webkit2gtk-driver on Ubuntu)
 */

describe("AgentsManager App Launch", () => {
  it("should display the app window", async () => {
    // The WebDriver session opens the Tauri app automatically via tauri:options.
    // Verify we have an active window.
    const title = await browser.getTitle();
    console.log("Window title:", title);
    // Title may be empty or "Agents Manager" depending on config; just assert it doesn't throw.
    expect(typeof title).toBe("string");
  });

  it("should render the main container", async () => {
    // Wait for the root React mount point to appear
    const root = await $("#root");
    await root.waitForExist({ timeout: 15_000 });
    expect(await root.isExisting()).toBe(true);
  });

  it("should take a baseline screenshot", async () => {
    // Save screenshot for visual reference / future regression baseline
    const screenshot = await browser.saveScreenshot("./screenshots/app-launch.png");
    expect(screenshot).toBeTruthy();
  });
});
