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
    description: 'Member ID to add as a member',
    example: 'member-123-abc',
  })
  @IsString()
  @IsNotEmpty()
  memberId: string;

  @ApiProperty({
    description: 'Role to assign to the member',
    enum: GroupRole,
    example: GroupRole.SACCO_ADMIN,
  })
  @IsEnum(GroupRole)
  role: GroupRole;
}
