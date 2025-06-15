import { Injectable } from '@nestjs/common';
import { ContextAwareService, ServiceContext } from './context.service';

/**
 * Service registry for managing context-aware services
 */
@Injectable()
export class ServiceRegistry {
  private services: Map<string, ContextAwareService> = new Map();

  registerService(name: string, service: ContextAwareService): void {
    this.services.set(name, service);
  }

  getService(name: string): ContextAwareService | undefined {
    return this.services.get(name);
  }

  getAvailableServices(): string[] {
    return Array.from(this.services.keys());
  }

  /**
   * Get services available to user in specific context
   */
  getServicesForContext(context: ServiceContext): string[] {
    const availableServices: string[] = [];

    for (const [serviceName, service] of this.services) {
      const operations = service.getServiceOperations();

      // Check if any operation is available in current scope
      const hasAvailableOperation = Object.values(operations).some(
        (operation) => operation.allowedScopes.includes(context.scope),
      );

      if (hasAvailableOperation) {
        availableServices.push(serviceName);
      }
    }

    return availableServices;
  }
}
