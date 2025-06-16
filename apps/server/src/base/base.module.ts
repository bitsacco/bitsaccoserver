import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';

import { CommonModule } from '@/common';
import {
  ApprovalWorkflow,
  ApprovalWorkflowSchema,
  SegregationRule,
  SegregationRuleSchema,
  TransactionLimit,
  TransactionLimitSchema,
  ComplianceEvent,
  ComplianceEventSchema,
  RegulatoryReport,
  RegulatoryReportSchema,
} from '@/common/schemas';

import {
  ComplianceService,
  GovernanceService,
  RiskManagementService,
  SegregationService,
} from './';
import { MakerCheckerService } from './maker-checker.service';

@Module({
  imports: [
    CommonModule,
    MongooseModule.forFeature([
      { name: ApprovalWorkflow.name, schema: ApprovalWorkflowSchema },
      { name: SegregationRule.name, schema: SegregationRuleSchema },
      { name: TransactionLimit.name, schema: TransactionLimitSchema },
      { name: ComplianceEvent.name, schema: ComplianceEventSchema },
      { name: RegulatoryReport.name, schema: RegulatoryReportSchema },
    ]),
  ],
  providers: [
    ComplianceService,
    GovernanceService,
    RiskManagementService,
    SegregationService,
    MakerCheckerService,
  ],
  exports: [
    ComplianceService,
    GovernanceService,
    RiskManagementService,
    SegregationService,
    MakerCheckerService,
  ],
})
export class BaseModule {}
