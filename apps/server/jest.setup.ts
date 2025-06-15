// Global test setup and mocks

// Mock the entire common module before any imports
beforeAll(() => {
  // Clear all module caches
  jest.resetModules();
});

// Mock problematic schemas at the module level
jest.mock('./src/common/schemas/compliance.schema', () => {
  const createMockClass = (name: string) => {
    class MockClass {}
    Object.defineProperty(MockClass, 'name', {
      value: name,
      configurable: true,
      writable: false,
    });
    return MockClass;
  };

  return {
    __esModule: true,
    AuditTrail: createMockClass('AuditTrail'),
    AuditTrailDocument: {},
    AuditTrailSchema: {},
    ApprovalWorkflow: createMockClass('ApprovalWorkflow'),
    ApprovalWorkflowDocument: {},
    ApprovalWorkflowSchema: {},
    ApprovalStatus: {
      PENDING: 'PENDING',
      APPROVED: 'APPROVED',
      REJECTED: 'REJECTED',
      EXPIRED: 'EXPIRED',
      CANCELLED: 'CANCELLED',
    },
    WorkflowType: {
      TRANSACTION_APPROVAL: 'TRANSACTION_APPROVAL',
      USER_ROLE_CHANGE: 'USER_ROLE_CHANGE',
      LIMIT_INCREASE: 'LIMIT_INCREASE',
      CONFIGURATION_CHANGE: 'CONFIGURATION_CHANGE',
    },
    RiskLevel: {
      LOW: 'LOW',
      MEDIUM: 'MEDIUM',
      HIGH: 'HIGH',
      CRITICAL: 'CRITICAL',
    },
  };
});

jest.mock('./src/common/schemas/sacco.schema', () => {
  const createMockClass = (name: string) => {
    class MockClass {}
    Object.defineProperty(MockClass, 'name', {
      value: name,
      configurable: true,
      writable: false,
    });
    return MockClass;
  };

  return {
    __esModule: true,
    Sacco: createMockClass('Sacco'),
    SaccoDocument: {},
    SaccoSchema: {},
    Chama: createMockClass('Chama'),
    ChamaDocument: {},
    ChamaSchema: {},
  };
});

// Mock the services that use these schemas
jest.mock('./src/common/services/audit.service', () => ({
  __esModule: true,
  AuditService: jest.fn().mockImplementation(() => ({
    logEvent: jest.fn(),
  })),
}));

jest.mock('./src/common/services/sacco.service', () => ({
  __esModule: true,
  SaccoService: jest.fn().mockImplementation(() => ({
    // Add any methods that are used in tests
  })),
}));
