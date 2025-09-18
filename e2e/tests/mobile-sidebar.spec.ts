import { test, expect } from '@playwright/test';

test.describe('Mobile Sidebar Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the login page and authenticate
    await page.goto('/');

    // Use the same credentials as working auth tests
    await page.fill('input[name="phone"]', '+254700123456');
    await page.fill('input[name="pin"]', '123456');

    // Monitor network requests to capture the login response
    const loginResponsePromise = page.waitForResponse(
      response => response.url().includes('nestjs_login_action') && response.status() === 302,
      { timeout: 20000 }
    );

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for the login response
    await loginResponsePromise;

    // Wait for successful login to dashboard
    await Promise.race([
      page.waitForURL('**/dashboard', { timeout: 15000 }),
      page.waitForFunction(() => window.location.pathname.includes('/dashboard'), { timeout: 15000 })
    ]);

    // Set viewport to mobile size
    await page.setViewportSize({ width: 375, height: 667 });
  });

  test('should show hamburger menu on mobile', async ({ page }) => {
    // Verify hamburger button is visible on mobile
    const hamburgerButton = page.locator('button[aria-label="Open sidebar"]');
    await expect(hamburgerButton).toBeVisible();

    // Verify desktop sidebar is hidden
    const desktopSidebar = page.locator('.lg\\:flex.lg\\:w-64');
    await expect(desktopSidebar).not.toBeVisible();
  });

  test('should open mobile sidebar when hamburger is clicked', async ({ page }) => {
    // Click the hamburger button
    const hamburgerButton = page.locator('button[aria-label="Open sidebar"]');
    await hamburgerButton.click();

    // Wait for mobile sidebar to appear
    await expect(page.locator('.fixed.inset-0.z-40')).toBeVisible({ timeout: 5000 });

    // Verify sidebar content is visible
    await expect(page.locator('text=Bitsacco')).toBeVisible();
    await expect(page.locator('a[href="/dashboard"]')).toBeVisible();
    await expect(page.locator('a[href="/members"]')).toBeVisible();
  });

  test('should close mobile sidebar when backdrop is clicked', async ({ page }) => {
    // Open the sidebar first
    const hamburgerButton = page.locator('button[aria-label="Open sidebar"]');
    await hamburgerButton.click();

    // Wait for sidebar to open
    await expect(page.locator('.fixed.inset-0.z-40')).toBeVisible();

    // Click the backdrop
    await page.locator('.fixed.inset-0.bg-gray-600').click();

    // Verify sidebar is closed
    await expect(page.locator('.fixed.inset-0.z-40')).not.toBeVisible();
  });

  test('should close mobile sidebar when close button is clicked', async ({ page }) => {
    // Open the sidebar first
    const hamburgerButton = page.locator('button[aria-label="Open sidebar"]');
    await hamburgerButton.click();

    // Wait for sidebar to open
    await expect(page.locator('.fixed.inset-0.z-40')).toBeVisible();

    // Click the close button
    const closeButton = page.locator('button[type="button"]').filter({ hasText: 'Close sidebar' });
    await closeButton.click();

    // Verify sidebar is closed
    await expect(page.locator('.fixed.inset-0.z-40')).not.toBeVisible();
  });

  test('should navigate using mobile sidebar links', async ({ page }) => {
    // Open the sidebar
    const hamburgerButton = page.locator('button[aria-label="Open sidebar"]');
    await hamburgerButton.click();

    // Wait for sidebar to open
    await expect(page.locator('.fixed.inset-0.z-40')).toBeVisible();

    // Click on Members link
    await page.locator('a[href="/members"]').click();

    // Verify navigation to members page
    await expect(page.locator('main h1').filter({ hasText: 'Members' }).first()).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/members');

    // Verify mobile sidebar is closed after navigation
    await expect(page.locator('.fixed.inset-0.z-40')).not.toBeVisible();
  });
});