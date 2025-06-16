import {
  Injectable,
  NotFoundException,
  ConflictException,
  Logger,
} from '@nestjs/common';
import { Model } from 'mongoose';
import { InjectModel } from '@nestjs/mongoose';
import { GroupRole } from '@bitsaccoserver/types';
import {
  OrganizationDocument,
  OrganizationMember,
  OrganizationMemberDocument,
} from './organization.schema';
import {
  CreateOrganizationDto,
  UpdateOrganizationDto,
} from './organization.dto';

@Injectable()
export class OrganizationService {
  private readonly logger = new Logger(OrganizationService.name);

  constructor(
    @InjectModel(OrganizationDocument.name)
    private organizationModel: Model<OrganizationDocument>,
    @InjectModel(OrganizationMember.name)
    private organizationMemberModel: Model<OrganizationMemberDocument>,
  ) {}

  async create(
    createOrganizationDto: CreateOrganizationDto,
    memberId: string,
    memberEmail: string,
  ): Promise<OrganizationDocument> {
    // Check if organization name already exists
    const existingOrg = await this.organizationModel.findOne({
      name: createOrganizationDto.name,
    });
    if (existingOrg) {
      throw new ConflictException('Organization name already exists');
    }

    // Create organization
    const organization = new this.organizationModel({
      ...createOrganizationDto,
      ownerId: memberId,
      ownerEmail: memberEmail,
      limits: {
        maxApiKeys: 10,
        maxMonthlyVolume: 1000000,
        maxDailyRequests: 10000,
      },
      settings: {},
    });

    const savedOrg = await organization.save();

    // Add owner as admin member
    try {
      await this.addMember(
        savedOrg._id.toString(),
        memberId,
        GroupRole.ORG_ADMIN,
        memberId,
      );
    } catch (error) {
      this.logger.debug(
        `Failed to add owner as member during organization creation: ${JSON.stringify(
          {
            organizationId: savedOrg._id.toString(),
            memberId,
            error: error.message,
          },
        )}`,
      );
      throw error; // Re-throw to ensure organization creation fails if membership fails
    }

    return savedOrg;
  }

  async findAll(memberId: string): Promise<OrganizationDocument[]> {
    // Find all organizations where member is a member
    const memberships = await this.organizationMemberModel
      .find({ memberId, isActive: true })
      .select('organizationId');

    const orgIds = memberships.map((m) => m.organizationId);

    return this.organizationModel.find({
      _id: { $in: orgIds },
      isActive: true,
    });
  }

  async findOne(id: string): Promise<OrganizationDocument> {
    const organization = await this.organizationModel.findById(id);
    if (!organization) {
      throw new NotFoundException('Organization not found');
    }
    return organization;
  }

  async update(
    id: string,
    updateOrganizationDto: UpdateOrganizationDto,
  ): Promise<OrganizationDocument> {
    const organization = await this.organizationModel.findByIdAndUpdate(
      id,
      updateOrganizationDto,
      { new: true },
    );

    if (!organization) {
      throw new NotFoundException('Organization not found');
    }

    return organization;
  }

  async delete(id: string): Promise<void> {
    const result = await this.organizationModel.findByIdAndUpdate(
      id,
      { isActive: false },
      { new: true },
    );

    if (!result) {
      throw new NotFoundException('Organization not found');
    }

    // Deactivate all memberships
    await this.organizationMemberModel.updateMany(
      { organizationId: id },
      { isActive: false },
    );
  }

  async addMember(
    organizationId: string,
    memberId: string,
    role: GroupRole,
    invitedBy: string,
  ): Promise<OrganizationMember> {
    this.logger.debug(
      `Adding member - organizationId: ${organizationId}, memberId: ${memberId}, role: ${role}, invitedBy: ${invitedBy}`,
    );

    // Validate input parameters
    if (!organizationId || !memberId || !role || !invitedBy) {
      this.logger.error(
        `Missing required parameters for addMember: ${JSON.stringify({ organizationId, memberId, role, invitedBy })}`,
      );
      throw new Error('Missing required parameters for adding member');
    }

    // Check if member is already a member
    const existingMember = await this.organizationMemberModel.findOne({
      organizationId,
      memberId,
    });

    if (existingMember) {
      if (existingMember.isActive) {
        throw new ConflictException(
          'Member is already a member of this organization',
        );
      }
      // Reactivate existing membership
      existingMember.isActive = true;
      existingMember.role = role;
      existingMember.joinedAt = new Date();
      return existingMember.save();
    }

    const member = new this.organizationMemberModel({
      organizationId,
      memberId,
      role,
      invitedBy,
      invitedAt: new Date(),
      joinedAt: new Date(),
    });

    return member.save();
  }

  async getMembers(organizationId: string): Promise<OrganizationMember[]> {
    return this.organizationMemberModel.find({
      organizationId,
      isActive: true,
    });
  }

  async removeMember(organizationId: string, memberId: string): Promise<void> {
    const result = await this.organizationMemberModel.findOneAndUpdate(
      { organizationId, memberId },
      { isActive: false },
      { new: true },
    );

    if (!result) {
      throw new NotFoundException('Member not found');
    }
  }

  async updateMemberRole(
    organizationId: string,
    memberId: string,
    role: GroupRole,
  ): Promise<OrganizationMember> {
    const member = await this.organizationMemberModel.findOneAndUpdate(
      { organizationId, memberId, isActive: true },
      { role },
      { new: true },
    );

    if (!member) {
      throw new NotFoundException('Member not found');
    }

    return member;
  }
}
