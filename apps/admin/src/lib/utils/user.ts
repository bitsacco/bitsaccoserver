import { User } from '@bitsaccoserver/types';

/**
 * Get a display name for a user with fallback logic
 */
export function getUserDisplayName(user: User | null | undefined): string {
  if (!user) return 'User';

  // If user has a pre-computed name field, use it
  if (user.name && user.name.trim()) {
    return user.name.trim();
  }

  // If user has firstName and/or lastName, combine them
  if (user.firstName || user.lastName) {
    const fullName = `${user.firstName || ''} ${user.lastName || ''}`.trim();
    if (fullName) {
      return fullName;
    }
  }

  // Fall back to email username
  if (user.email) {
    return user.email.split('@')[0];
  }

  // Final fallback
  return 'User';
}

/**
 * Get the first available initial for avatar display
 */
export function getUserInitial(user: User | null | undefined): string {
  if (!user) return 'U';

  // Try firstName first
  if (user.firstName?.charAt(0)) {
    return user.firstName.charAt(0).toUpperCase();
  }

  // Try name field
  if (user.name?.charAt(0)) {
    return user.name.charAt(0).toUpperCase();
  }

  // Try email
  if (user.email?.charAt(0)) {
    return user.email.charAt(0).toUpperCase();
  }

  return 'U';
}
