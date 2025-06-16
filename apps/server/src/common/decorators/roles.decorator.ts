import { SetMetadata } from '@nestjs/common';
import { ServiceRole, GroupRole } from '../types';

export const ROLES_KEY = 'roles';
export const Roles = (...roles: (ServiceRole | GroupRole)[]) =>
  SetMetadata(ROLES_KEY, roles);
