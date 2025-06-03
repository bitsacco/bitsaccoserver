/**
 * Utility function to fetch data with authentication through the API proxy
 * Handles token refresh and auth redirects
 */
export async function fetchWithAuth(
  path: string,
  options: RequestInit = {},
): Promise<Response> {
  const headers = new Headers(options.headers || {});

  // Add content type if not present for non-GET requests
  if (
    !headers.has('Content-Type') &&
    options.method &&
    options.method !== 'GET'
  ) {
    headers.set('Content-Type', 'application/json');
  }

  // Add authorization token from localStorage if available
  const token =
    typeof localStorage !== 'undefined'
      ? localStorage.getItem('access-token')
      : null;

  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  // Ensure path starts with /api if not already
  const apiPath = path.startsWith('/api')
    ? path
    : `/api${path.startsWith('/') ? path : `/${path}`}`;

  // Set up the request
  const request = new Request(apiPath, {
    ...options,
    headers,
    credentials: 'include', // Include cookies for auth
  });

  // Make the request
  const response = await fetch(request);

  // Handle auth errors
  if (response.status === 401 && typeof window !== 'undefined') {
    // Clear tokens on unauthorized
    localStorage.removeItem('access-token');
    localStorage.removeItem('refresh-token');

    // Redirect to login if unauthorized
    window.location.href = `/auth/sign-in?returnTo=${encodeURIComponent(window.location.pathname)}`;
    throw new Error('Unauthorized');
  }

  return response;
}
