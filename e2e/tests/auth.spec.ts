import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the login page before each test
    await page.goto('/');
  });

  test('should display login form', async ({ page }) => {
    // Check that login form elements are present (using NestJS form field names)
    await expect(page.locator('input[name="phone"]')).toBeVisible();
    await expect(page.locator('input[name="pin"]')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should successfully login super admin and access dashboard', async ({ page }) => {
    // Fill in super admin credentials
    await page.fill('input[name="phone"]', '+254700123456');
    await page.fill('input[name="pin"]', '123456');

    // Monitor network requests to capture the login response
    const loginResponsePromise = page.waitForResponse(
      response => response.url().includes('nestjs_login_action') && response.status() === 302
    );

    // Submit the form
    await page.click('button[type="submit"]');

    // Wait for the login response
    const loginResponse = await loginResponsePromise;

    // Verify we get a redirect response
    expect(loginResponse.status()).toBe(302);

    // Wait for navigation to dashboard with longer timeout and better waiting strategy
    await Promise.race([
      page.waitForURL('**/dashboard', { timeout: 15000 }),
      page.waitForFunction(() => window.location.pathname.includes('/dashboard'), { timeout: 15000 })
    ]);

    // Verify we're on the dashboard page
    expect(page.url()).toContain('/dashboard');

    // Check that we don't see "Access Denied"
    const pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Verify cookies are set correctly
    const cookies = await page.context().cookies();

    // Check for auth_token cookie (should be readable by client)
    const authTokenCookie = cookies.find(cookie => cookie.name === 'auth_token');
    expect(authTokenCookie, 'auth_token cookie should be set').toBeDefined();
    expect(authTokenCookie?.value, 'auth_token should have a value').toBeTruthy();
    expect(authTokenCookie?.httpOnly, 'auth_token should NOT be HttpOnly for client access').toBeFalsy();
    expect(authTokenCookie?.sameSite, 'auth_token should have SameSite=Strict').toBe('Strict');

    // Check for refresh_token cookie (should be HttpOnly)
    const refreshTokenCookie = cookies.find(cookie => cookie.name === 'refresh_token');
    expect(refreshTokenCookie, 'refresh_token cookie should be set').toBeDefined();
    expect(refreshTokenCookie?.value, 'refresh_token should have a value').toBeTruthy();
    expect(refreshTokenCookie?.httpOnly, 'refresh_token should be HttpOnly for security').toBeTruthy();
    expect(refreshTokenCookie?.sameSite, 'refresh_token should have SameSite=Strict').toBe('Strict');
  });

  test('should maintain authentication cookies and validate persistence', async ({ page }) => {
    // First, login
    await page.fill('input[name="phone"]', '+254700123456');
    await page.fill('input[name="pin"]', '123456');

    // Monitor network requests to capture the login response (like working auth tests)
    const loginResponsePromise = page.waitForResponse(
      response => response.url().includes('nestjs_login_action') && response.status() === 302,
      { timeout: 20000 }
    );

    await page.click('button[type="submit"]');

    // Wait for the login response
    await loginResponsePromise;

    // Wait for navigation to dashboard with better waiting strategy
    await Promise.race([
      page.waitForURL('**/dashboard', { timeout: 15000 }),
      page.waitForFunction(() => window.location.pathname.includes('/dashboard'), { timeout: 15000 })
    ]);

    // Verify we're authenticated
    let pageContent = await page.textContent('body');
    expect(pageContent).not.toContain('Access Denied');

    // Check that cookies are present and properly configured
    let cookies = await page.context().cookies();
    let authTokenCookie = cookies.find(cookie => cookie.name === 'auth_token');
    let refreshTokenCookie = cookies.find(cookie => cookie.name === 'refresh_token');

    expect(authTokenCookie, 'auth_token should be present').toBeDefined();
    expect(refreshTokenCookie, 'refresh_token should be present').toBeDefined();

    // Verify cookie attributes for persistence
    expect(authTokenCookie?.path).toBe('/');
    expect(authTokenCookie?.sameSite).toBe('Strict');
    expect(authTokenCookie?.httpOnly).toBeFalsy(); // Should be readable by client

    expect(refreshTokenCookie?.path).toBe('/');
    expect(refreshTokenCookie?.sameSite).toBe('Strict');
    expect(refreshTokenCookie?.httpOnly).toBeTruthy(); // Should be HttpOnly for security

    // Verify tokens have actual values (not empty)
    expect(authTokenCookie?.value.length).toBeGreaterThan(10);
    expect(refreshTokenCookie?.value.length).toBeGreaterThan(10);

    // Verify client-side cookie access (auth_token should be accessible via document.cookie)
    const clientCookies = await page.evaluate(() => document.cookie);
    expect(clientCookies).toContain('auth_token=');
    expect(clientCookies).not.toContain('refresh_token='); // refresh_token should be HttpOnly
  });

  test('should handle invalid credentials gracefully', async ({ page }) => {
    // Fill in invalid credentials
    await page.fill('input[name="phone"]', '+254700000000');
    await page.fill('input[name="pin"]', '000000');

    // Submit the form
    await page.click('button[type="submit"]');

    // Should stay on login page or show error
    // Note: This depends on how your error handling works
    await page.waitForTimeout(2000); // Wait for potential error message

    // Should not have auth cookies
    const cookies = await page.context().cookies();
    const authTokenCookie = cookies.find(cookie => cookie.name === 'auth_token');
    expect(authTokenCookie).toBeUndefined();
  });

  test('should verify cookie security attributes', async ({ page }) => {
    // Login first
    await page.fill('input[name="phone"]', '+254700123456');
    await page.fill('input[name="pin"]', '123456');

    // Monitor network requests to capture the login response (like working auth tests)
    const loginResponsePromise = page.waitForResponse(
      response => response.url().includes('nestjs_login_action') && response.status() === 302,
      { timeout: 20000 }
    );

    await page.click('button[type="submit"]');

    // Wait for the login response
    await loginResponsePromise;

    // Wait for navigation to dashboard with better waiting strategy
    await Promise.race([
      page.waitForURL('**/dashboard', { timeout: 15000 }),
      page.waitForFunction(() => window.location.pathname.includes('/dashboard'), { timeout: 15000 })
    ]);

    // Get cookies and verify security attributes
    const cookies = await page.context().cookies();

    const authTokenCookie = cookies.find(cookie => cookie.name === 'auth_token');
    const refreshTokenCookie = cookies.find(cookie => cookie.name === 'refresh_token');

    // Verify auth_token attributes
    expect(authTokenCookie?.path).toBe('/');
    expect(authTokenCookie?.sameSite).toBe('Strict');
    expect(authTokenCookie?.httpOnly).toBeFalsy(); // Should be readable by client

    // Verify refresh_token attributes
    expect(refreshTokenCookie?.path).toBe('/');
    expect(refreshTokenCookie?.sameSite).toBe('Strict');
    expect(refreshTokenCookie?.httpOnly).toBeTruthy(); // Should be HttpOnly for security
  });
});