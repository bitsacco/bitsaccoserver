import { test, expect } from '@playwright/test';

test.describe('Dashboard Navigation', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to login and authenticate first
    await page.goto('/');

    // Use the same credentials as the working auth tests
    await page.fill('input[name="phone"]', '+254700123456');
    await page.fill('input[name="pin"]', '123456');

    // Monitor network requests to capture the login response (like working auth tests)
    const loginResponsePromise = page.waitForResponse(
      response => response.url().includes('nestjs_login_action') && response.status() === 302,
      { timeout: 20000 }
    );

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for the login response
    await loginResponsePromise;

    // Wait for successful login to dashboard using the same pattern as auth tests
    await Promise.race([
      page.waitForURL('**/dashboard', { timeout: 15000 }),
      page.waitForFunction(() => window.location.pathname.includes('/dashboard'), { timeout: 15000 })
    ]);
  });

  test('should navigate to members page', async ({ page }) => {
    // Navigate to members page
    await page.goto('/members');

    // Wait for the specific page content to load - members page has h1 with "Members"
    await expect(page.locator('h1').filter({ hasText: 'Members' })).toBeVisible({ timeout: 15000 });

    // Verify we're on the members page
    expect(page.url()).toContain('/members');

    // Check that we don't see "Access Denied"
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Verify we can see the page description
    await expect(page.locator('p').filter({ hasText: 'Member management and registration' })).toBeVisible();
  });

  test('should navigate to groups page', async ({ page }) => {
    // Navigate to groups page
    await page.goto('/groups');

    // Wait for the specific page content to load - groups page has h1 with "Groups"
    await expect(page.locator('h1').filter({ hasText: 'Groups' })).toBeVisible({ timeout: 15000 });

    // Verify we're on the groups page
    expect(page.url()).toContain('/groups');

    // Check that we don't see "Access Denied"
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Verify we can see the page description
    await expect(page.locator('p').filter({ hasText: 'Group management and organization' })).toBeVisible();
  });

  test('should navigate to shares page', async ({ page }) => {
    // Navigate to shares page
    await page.goto('/shares');

    // Wait for the specific page content to load - shares page has h1 with "Shares"
    await expect(page.locator('h1').filter({ hasText: 'Shares' })).toBeVisible({ timeout: 15000 });

    // Verify we're on the shares page
    expect(page.url()).toContain('/shares');

    // Check that we don't see "Access Denied"
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Verify we can see the page description
    await expect(page.locator('p').filter({ hasText: 'Manage share offers, purchases, and transfers within your SACCO' })).toBeVisible();
  });

  test('should navigate to settings page', async ({ page }) => {
    // Navigate to settings page
    await page.goto('/settings');

    // Wait for the specific page content to load - settings page has h1 with "Settings"
    await expect(page.locator('h1').filter({ hasText: 'Settings' })).toBeVisible({ timeout: 15000 });

    // Verify we're on the settings page
    expect(page.url()).toContain('/settings');

    // Check that we don't see "Access Denied"
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Verify we can see the page description
    await expect(page.locator('p').filter({ hasText: 'Manage your account settings and preferences' })).toBeVisible();
  });

  test('should navigate between dashboard pages using sidebar links', async ({ page }) => {
    // Start from dashboard - verify we can see dashboard content
    expect(page.url()).toContain('/dashboard');
    await expect(page.locator('h2').filter({ hasText: 'Key Performance Indicators' })).toBeVisible({ timeout: 10000 });

    // Navigate to members via sidebar
    await page.click('a[href="/members"]');
    await expect(page.locator('h1').filter({ hasText: 'Members' })).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/members');

    // Navigate to groups via sidebar
    await page.click('a[href="/groups"]');
    await expect(page.locator('h1').filter({ hasText: 'Groups' })).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/groups');

    // Navigate to shares via sidebar
    await page.click('a[href="/shares"]');
    await expect(page.locator('h1').filter({ hasText: 'Shares' })).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/shares');

    // Navigate to settings via sidebar
    await page.click('a[href="/settings"]');
    await expect(page.locator('h1').filter({ hasText: 'Settings' })).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/settings');

    // Navigate back to dashboard
    await page.click('a[href="/dashboard"]');
    await expect(page.locator('h2').filter({ hasText: 'Key Performance Indicators' })).toBeVisible({ timeout: 15000 });
    expect(page.url()).toContain('/dashboard');

    // Verify all navigation worked without access denied
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');
  });
});