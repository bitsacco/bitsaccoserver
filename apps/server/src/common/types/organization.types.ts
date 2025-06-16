// Organization and SACCO management types

export interface CreateOrganizationDto {
  name: string;
  type: 'sacco' | 'chama';
  description?: string;
  registrationNumber?: string;
  taxId?: string;
  country: string;
  region?: string;
  address?: {
    street?: string;
    city?: string;
    state?: string;
    postalCode?: string;
    country: string;
  };
  contactInfo?: {
    email?: string;
    phone?: string;
    website?: string;
  };
  settings?: {
    currency: string;
    timezone: string;
    language: string;
  };
}

export interface UpdateOrganizationDto {
  name?: string;
  description?: string;
  registrationNumber?: string;
  taxId?: string;
  country?: string;
  region?: string;
  address?: {
    street?: string;
    city?: string;
    state?: string;
    postalCode?: string;
    country: string;
  };
  contactInfo?: {
    email?: string;
    phone?: string;
    website?: string;
  };
  settings?: {
    currency?: string;
    timezone?: string;
    language?: string;
  };
  status?: 'active' | 'inactive' | 'suspended';
}

export interface AddMemberDto {
  userId: string;
  role: string; // Will reference GroupRole enum
  permissions?: string[];
  startDate?: Date;
  membershipType?: 'full' | 'associate' | 'honorary';
  membershipFee?: number;
  notes?: string;
}
