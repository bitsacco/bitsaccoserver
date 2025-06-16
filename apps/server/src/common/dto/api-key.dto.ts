import {
  IsString,
  IsOptional,
  IsArray,
  IsDate,
  IsBoolean,
  IsDateString,
  IsNotEmpty,
  IsObject,
} from 'class-validator';
import { Type } from 'class-transformer';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

export class CreateApiKeyDto {
  @ApiProperty({ description: 'API key name' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiPropertyOptional({ description: 'API key description' })
  @IsString()
  @IsOptional()
  description?: string;

  @ApiPropertyOptional({
    description: 'Services this key can access',
    default: [],
  })
  @IsArray()
  @IsOptional()
  serviceIds?: string[];

  @ApiPropertyOptional({
    description: 'Permissions for this key',
    default: [],
  })
  @IsArray()
  @IsOptional()
  permissions?: string[];

  @ApiPropertyOptional({ description: 'Rate limits for this key' })
  @IsObject()
  @IsOptional()
  limits?: {
    requestsPerMinute?: number;
    requestsPerDay?: number;
    monthlyVolume?: number;
  };

  @ApiPropertyOptional({ description: 'Allowed IP addresses' })
  @IsArray()
  @IsOptional()
  allowedIps?: string[];

  @ApiPropertyOptional({ description: 'Allowed domains' })
  @IsArray()
  @IsOptional()
  allowedDomains?: string[];

  @ApiPropertyOptional({ description: 'Expiration date for the key' })
  @IsDateString()
  @IsOptional()
  expiresAt?: Date;
}

export class UpdateApiKeyDto {
  @ApiProperty({ description: 'Name for the API key', required: false })
  @IsOptional()
  @IsString()
  name?: string;

  @ApiProperty({ description: 'Description of the API key', required: false })
  @IsOptional()
  @IsString()
  description?: string;

  @ApiProperty({
    description: 'Roles assigned to the API key',
    required: false,
  })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  roles?: string[];

  @ApiProperty({ description: 'Permissions for the API key', required: false })
  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  permissions?: string[];

  @ApiProperty({
    description: 'Whether the API key is active',
    required: false,
  })
  @IsOptional()
  @IsBoolean()
  isActive?: boolean;

  @ApiProperty({
    description: 'Expiration date for the API key',
    required: false,
  })
  @IsOptional()
  @Type(() => Date)
  @IsDate()
  expiresAt?: Date;
}
