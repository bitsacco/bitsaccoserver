import {
  IsString,
  IsNotEmpty,
  IsOptional,
  IsObject,
  IsEnum,
} from 'class-validator';
import { ApiProperty, ApiPropertyOptional, PartialType } from '@nestjs/swagger';
import { GroupRole } from '@/common';

export class CreateOrganizationDto {
  @ApiProperty({ description: 'Organization name' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiProperty({ description: 'Country where organization is located' })
  @IsString()
  @IsNotEmpty()
  country: string;

  @ApiPropertyOptional({ description: 'Organization description' })
  @IsString()
  @IsOptional()
  description?: string;

  @ApiPropertyOptional({ description: 'KYB details' })
  @IsObject()
  @IsOptional()
  kybDetails?: {
    businessRegistrationNumber?: string;
    taxId?: string;
    businessAddress?: string;
    businessType?: string;
  };
}

export class UpdateOrganizationDto extends PartialType(CreateOrganizationDto) {}

export class AddMemberDto {
  @ApiProperty({
    description: 'User ID to add as a member',
    example: 'user-123-abc',
  })
  @IsString()
  @IsNotEmpty()
  userId: string;

  @ApiProperty({
    description: 'Role to assign to the user',
    enum: GroupRole,
    example: GroupRole.SACCO_ADMIN,
  })
  @IsEnum(GroupRole)
  role: GroupRole;
}
