import type { Icon } from '@phosphor-icons/react/dist/lib/types';
import {
  HouseLine,
  ChartBar,
  GearSix,
  Heart,
  User,
  Users,
  ShieldCheck,
} from '@phosphor-icons/react';

export const navIcons = {
  home: HouseLine,
  chart: ChartBar,
  gear: GearSix,
  heart: Heart,
  shield: ShieldCheck,
  user: User,
  users: Users,
} as Record<string, Icon>;
