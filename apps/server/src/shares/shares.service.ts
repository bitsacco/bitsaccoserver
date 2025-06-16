import {
  Injectable,
  Logger,
  BadRequestException,
  NotFoundException,
} from '@nestjs/common';
import { Model } from 'mongoose';
import { InjectModel } from '@nestjs/mongoose';
import {
  OfferSharesDto,
  BuySharesDto,
  TransferSharesDto,
  UpdateSharesDto,
  UserSharesDto,
  FindSharesTxDto,
  MetricsService,
  ContextAwareService,
  SharesDocument,
  SharesOfferDocument,
  SharesTxStatus,
  PermissionService,
  SaccoService,
  Permission,
  PermissionScope,
  ServiceOperation,
  ServiceContext,
  PaginationDto,
  AuthenticatedUser,
  RiskLevel,
} from '../common';

@Injectable()
export class SharesService extends ContextAwareService {
  private readonly logger = new Logger(SharesService.name);
  private readonly DEFAULT_PAGE = 1;
  private readonly DEFAULT_PAGE_SIZE = 10;

  constructor(
    @InjectModel(SharesDocument.name)
    private readonly sharesModel: Model<SharesDocument>,
    @InjectModel(SharesOfferDocument.name)
    private readonly sharesOfferModel: Model<SharesOfferDocument>,
    private readonly metricsService: MetricsService,
    protected permissionService: PermissionService,
    protected saccoService: SaccoService,
  ) {
    super(permissionService, saccoService);
    this.logger.log('SharesService created');
  }

  getServiceOperations(): Record<string, ServiceOperation> {
    return {
      purchaseShares: {
        name: 'purchaseShares',
        requiredPermissions: [Permission.SHARES_TRADE],
        allowedScopes: [PermissionScope.ORGANIZATION, PermissionScope.PERSONAL],
        description: 'Purchase shares in an offering',
        riskLevel: RiskLevel.MEDIUM,
        auditLevel: 'detailed',
      },
      sellShares: {
        name: 'sellShares',
        requiredPermissions: [Permission.SHARES_TRADE],
        allowedScopes: [PermissionScope.ORGANIZATION, PermissionScope.PERSONAL],
        requiresApproval: true,
        description: 'Sell owned shares',
        riskLevel: RiskLevel.MEDIUM,
        auditLevel: 'detailed',
      },
      viewShares: {
        name: 'viewShares',
        requiredPermissions: [Permission.SHARES_READ],
        allowedScopes: [
          PermissionScope.GLOBAL,
          PermissionScope.ORGANIZATION,
          PermissionScope.PERSONAL,
        ],
        description: 'View shares information',
        riskLevel: RiskLevel.LOW,
        auditLevel: 'basic',
      },
      createOffer: {
        name: 'createOffer',
        requiredPermissions: [Permission.SHARES_CREATE],
        allowedScopes: [PermissionScope.ORGANIZATION],
        requiresApproval: true,
        description: 'Create new shares offering',
        riskLevel: RiskLevel.HIGH,
        auditLevel: 'comprehensive',
      },
    };
  }

  protected async performOperation<T>(
    operationName: string,
    context: ServiceContext,
    operationData: any,
  ): Promise<T> {
    switch (operationName) {
      case 'createOffer':
        return this.createOffer(context, operationData) as Promise<T>;
      case 'viewOffers':
        return this.viewOffers(context) as Promise<T>;
      case 'buyShares':
        return this.buyShares(context, operationData) as Promise<T>;
      case 'transferShares':
        return this.transferSharesInternal(
          context,
          operationData,
        ) as Promise<T>;
      case 'updateShares':
        return this.updateSharesInternal(context, operationData) as Promise<T>;
      case 'viewShares':
        return this.viewShares(context, operationData) as Promise<T>;
      case 'viewAllShares':
        return this.viewAllShares(context, operationData) as Promise<T>;
      default:
        throw new BadRequestException(
          `Unsupported shares operation: ${operationName}`,
        );
    }
  }

  // Helper method to create mock context (should be replaced with real auth context)
  private createMockContext(): ServiceContext {
    const mockUser: AuthenticatedUser = {
      userId: 'current-user',
      sub: 'current-user',
      email: 'user@example.com',
      authMethod: 'jwt',
      serviceRole: 'MEMBER' as any,
      servicePermissions: [],
      currentOrganizationId: 'current-org',
      currentScope: PermissionScope.ORGANIZATION,
      groupMemberships: [],
      contextPermissions: [],
      permissions: [],
    };

    return {
      userId: 'current-user',
      organizationId: 'current-org',
      chamaId: undefined,
      scope: PermissionScope.ORGANIZATION,
      permissions: [],
      user: mockUser,
    };
  }

  // Public API methods for controller
  async offerShares(offerData: OfferSharesDto) {
    return this.performOperation(
      'createOffer',
      this.createMockContext(),
      offerData,
    );
  }

  async getSharesOffers() {
    return this.performOperation('viewOffers', this.createMockContext(), {});
  }

  async subscribeShares(buyData: BuySharesDto) {
    return this.performOperation(
      'buyShares',
      this.createMockContext(),
      buyData,
    );
  }

  async transferShares(transferData: TransferSharesDto) {
    return this.performOperation(
      'transferShares',
      this.createMockContext(),
      transferData,
    );
  }

  async updateShares(updateData: UpdateSharesDto) {
    return this.performOperation(
      'updateShares',
      this.createMockContext(),
      updateData,
    );
  }

  async userSharesTransactions(userData: UserSharesDto) {
    return this.performOperation(
      'viewShares',
      this.createMockContext(),
      userData,
    );
  }

  async allSharesTransactions() {
    return this.performOperation('viewAllShares', this.createMockContext(), {});
  }

  // Note: Making this private method accessible
  // The private method expects different signature, so we create a new public interface

  private async createOffer(
    context: ServiceContext,
    { quantity, availableFrom, availableTo }: OfferSharesDto,
  ) {
    const startTime = Date.now();
    let success = false;
    let errorType: string | undefined;

    try {
      if (quantity <= 0) {
        throw new BadRequestException(
          'Share offer quantity must be greater than zero',
        );
      }

      const offer = new this.sharesOfferModel({
        quantity,
        subscribedQuantity: 0,
        availableFrom: new Date(availableFrom),
        availableTo: availableTo ? new Date(availableTo) : undefined,
      });

      await offer.save();
      success = true;

      this.logger.log(`Created share offer with quantity ${quantity}`);
      return this.viewOffers(context);
    } catch (error) {
      errorType = error.message || 'Unknown error';
      this.logger.error(`Error offering shares: ${errorType}`, error.stack);
      throw error;
    } finally {
      this.metricsService.recordMetric('shares_offer', {
        quantity,
        success,
        duration: Date.now() - startTime,
        errorType,
      });
    }
  }

  private async viewOffers(_context: ServiceContext) {
    try {
      const offers = await this.sharesOfferModel
        .find({})
        .sort({ createdAt: -1 })
        .exec();

      const mappedOffers = offers.map((offer) => ({
        id: offer._id.toString(),
        quantity: offer.quantity,
        subscribedQuantity: offer.subscribedQuantity,
        availableFrom: offer.availableFrom.toISOString(),
        availableTo: offer.availableTo?.toISOString(),
        createdAt: offer.createdAt.toISOString(),
        updatedAt: offer.updatedAt.toISOString(),
      }));

      const totalOfferQuantity = offers.reduce(
        (sum, offer) => sum + offer.quantity,
        0,
      );
      const totalSubscribedQuantity = offers.reduce(
        (sum, offer) => sum + offer.subscribedQuantity,
        0,
      );

      return {
        offers: mappedOffers,
        totalOfferQuantity,
        totalSubscribedQuantity,
      };
    } catch (error) {
      this.logger.error(
        `Error getting shares offers: ${error.message}`,
        error.stack,
      );
      throw error;
    }
  }

  private async buyShares(
    context: ServiceContext,
    { userId, offerId, quantity }: BuySharesDto,
  ) {
    this.logger.debug(`Buying ${quantity} shares for user ${userId}`);
    const startTime = Date.now();
    let success = false;
    let errorType = '';

    try {
      const offer = await this.sharesOfferModel.findById(offerId).exec();
      if (!offer) {
        errorType = 'OFFER_NOT_FOUND';
        throw new NotFoundException(`Share offer with ID ${offerId} not found`);
      }

      const availableShares = offer.quantity - offer.subscribedQuantity;
      if (availableShares < quantity) {
        errorType = 'INSUFFICIENT_SHARES';
        throw new BadRequestException(
          `Not enough shares available. Requested: ${quantity}, Available: ${availableShares}`,
        );
      }

      const allOffers = await this.viewOffers(context);
      const totalSharesAvailable = allOffers.totalOfferQuantity;
      const maxSharesPerUser = Math.floor(totalSharesAvailable * 0.2);

      const userShares = await this.viewShares(context, {
        userId,
        pagination: { page: this.DEFAULT_PAGE, size: this.DEFAULT_PAGE_SIZE },
      });

      const currentHoldings = userShares.shareHoldings;
      const totalAfterSubscription = currentHoldings + quantity;

      const percentageOfTotal =
        (totalAfterSubscription / totalSharesAvailable) * 100;
      this.metricsService.recordSharesOwnershipMetric({
        userId,
        quantity: currentHoldings,
        percentageOfTotal,
        limitReached: percentageOfTotal >= 15,
      });

      if (totalAfterSubscription > maxSharesPerUser) {
        errorType = 'OWNERSHIP_LIMIT_EXCEEDED';
        throw new BadRequestException(
          `Subscription exceeds maximum allowed shares per user (20% of total). ` +
            `Current: ${currentHoldings}, Requested: ${quantity}, Maximum: ${maxSharesPerUser}`,
        );
      }

      const sharesTx = new this.sharesModel({
        userId,
        offerId,
        quantity,
        status: SharesTxStatus.PROPOSED,
      });

      await sharesTx.save();
      success = true;

      const result = await this.viewShares(context, {
        userId,
        pagination: { page: this.DEFAULT_PAGE, size: this.DEFAULT_PAGE_SIZE },
      });

      return result;
    } catch (error) {
      this.logger.error(`Error subscribing shares: ${error.message}`);
      errorType = errorType || 'UNKNOWN_ERROR';
      throw error;
    } finally {
      const duration = Date.now() - startTime;
      this.metricsService.recordSharesSubscriptionMetric({
        userId,
        offerId,
        quantity,
        success,
        duration,
        errorType: success ? undefined : errorType,
      });
    }
  }

  private async transferSharesInternal(
    context: ServiceContext,
    { sharesId, fromUserId, toUserId, quantity, reason }: TransferSharesDto,
  ) {
    const startTime = Date.now();
    let success = false;
    let errorType = '';

    try {
      const originShares = await this.sharesModel.findById(sharesId).exec();
      if (!originShares) {
        errorType = 'SHARES_NOT_FOUND';
        throw new NotFoundException('Shares transaction not found');
      }

      if (originShares.status !== SharesTxStatus.COMPLETE) {
        errorType = 'SHARES_NOT_AVAILABLE';
        throw new BadRequestException('Shares are not available to transfer');
      }

      if (originShares.quantity < quantity) {
        errorType = 'INSUFFICIENT_SHARES';
        throw new BadRequestException('Not enough shares to transfer');
      }

      const allOffers = await this.viewOffers(context);
      const totalSharesAvailable = allOffers.totalOfferQuantity;
      const maxSharesPerUser = Math.floor(totalSharesAvailable * 0.2);

      const recipientShares = await this.viewShares(context, {
        userId: toUserId,
        pagination: { page: this.DEFAULT_PAGE, size: this.DEFAULT_PAGE_SIZE },
      });

      const currentHoldings = recipientShares.shareHoldings;
      const totalAfterTransfer = currentHoldings + quantity;

      if (totalAfterTransfer > maxSharesPerUser) {
        errorType = 'OWNERSHIP_LIMIT_EXCEEDED';
        throw new BadRequestException(
          `Transfer exceeds maximum allowed shares per user (20% of total). ` +
            `Recipient Current: ${currentHoldings}, Transfer: ${quantity}, Maximum: ${maxSharesPerUser}`,
        );
      }

      const transfer = { fromUserId, toUserId, quantity, reason };

      await this.sharesModel
        .findByIdAndUpdate(sharesId, {
          quantity: originShares.quantity - quantity,
          transfer,
        })
        .exec();

      const newShares = new this.sharesModel({
        userId: toUserId,
        offerId: originShares.offerId,
        quantity,
        status: SharesTxStatus.COMPLETE,
        transfer,
      });

      await newShares.save();
      success = true;

      const result = await this.viewShares(context, {
        userId: fromUserId,
        pagination: { page: this.DEFAULT_PAGE, size: this.DEFAULT_PAGE_SIZE },
      });

      return result;
    } catch (error) {
      this.logger.error(`Error transferring shares: ${error.message}`);
      errorType = errorType || 'UNKNOWN_ERROR';
      throw error;
    } finally {
      const duration = Date.now() - startTime;
      this.metricsService.recordSharesTransferMetric({
        userId: fromUserId,
        fromUserId,
        toUserId,
        quantity,
        success,
        duration,
        errorType: success ? undefined : errorType,
      });
    }
  }

  private async updateSharesInternal(
    context: ServiceContext,
    { sharesId, updates }: UpdateSharesDto,
  ) {
    try {
      const originShares = await this.sharesModel.findById(sharesId).exec();
      if (!originShares) {
        throw new NotFoundException('Shares transaction not found');
      }

      const { quantity, status, transfer, offerId } = updates;

      const updatedShares = await this.sharesModel
        .findByIdAndUpdate(
          sharesId,
          {
            quantity: quantity !== undefined ? quantity : originShares.quantity,
            status: status !== undefined ? status : originShares.status,
            transfer: transfer ?? originShares.transfer,
            offerId: offerId ?? originShares.offerId,
          },
          { new: true },
        )
        .exec();

      if (
        status === SharesTxStatus.COMPLETE ||
        status === SharesTxStatus.APPROVED
      ) {
        const offer = await this.sharesOfferModel
          .findById(originShares.offerId)
          .exec();
        if (offer) {
          const newQuantity = offer.subscribedQuantity + originShares.quantity;
          await this.sharesOfferModel
            .findByIdAndUpdate(originShares.offerId, {
              subscribedQuantity: newQuantity,
            })
            .exec();

          this.logger.log(
            `Updated offer ${originShares.offerId} subscribed quantity to ${newQuantity}`,
          );
        }
      }

      return this.viewShares(context, {
        userId: updatedShares.userId,
        pagination: { page: this.DEFAULT_PAGE, size: this.DEFAULT_PAGE_SIZE },
      });
    } catch (error) {
      this.logger.error(`Error updating shares: ${error.message}`);
      throw error;
    }
  }

  private async viewShares(
    context: ServiceContext,
    { userId, pagination }: UserSharesDto,
  ) {
    try {
      const paginationParams = pagination || {
        page: this.DEFAULT_PAGE,
        size: this.DEFAULT_PAGE_SIZE,
      };

      const shares = await this.sharesModel
        .find({
          userId,
          status: { $ne: SharesTxStatus.UNRECOGNIZED },
        })
        .sort({ createdAt: -1 })
        .exec();

      const shareHoldings = shares
        .filter(
          (share) =>
            share.status === SharesTxStatus.COMPLETE ||
            share.status === SharesTxStatus.APPROVED,
        )
        .reduce((sum, share) => sum + share.quantity, 0);

      const transactions = await this.getPaginatedShareTx(
        { userId },
        paginationParams,
      );
      const offers = await this.viewOffers(context);

      return {
        userId,
        shareHoldings,
        shares: transactions,
        offers,
      };
    } catch (error) {
      this.logger.error(`Error getting user shares: ${error.message}`);
      throw error;
    }
  }

  private async viewAllShares(
    context: ServiceContext,
    { page: _page, size: _size }: PaginationDto,
  ) {
    try {
      const shares = await this.getPaginatedShareTx(null, {
        page: this.DEFAULT_PAGE,
        size: this.DEFAULT_PAGE_SIZE,
      });

      const offers = await this.viewOffers(context);

      return {
        shares,
        offers,
      };
    } catch (error) {
      this.logger.error(
        `Error getting all shares transactions: ${error.message}`,
      );
      throw error;
    }
  }

  async findSharesTransaction({ sharesId }: FindSharesTxDto) {
    try {
      const shares = await this.sharesModel.findById(sharesId).exec();
      if (!shares) {
        throw new NotFoundException('Shares transaction not found');
      }

      return {
        id: shares._id.toString(),
        userId: shares.userId,
        offerId: shares.offerId,
        quantity: shares.quantity,
        status: shares.status,
        transfer: shares.transfer,
        createdAt: shares.createdAt.toISOString(),
        updatedAt: shares.updatedAt.toISOString(),
      };
    } catch (error) {
      this.logger.error(`Error finding shares transaction: ${error.message}`);
      throw error;
    }
  }

  /**
   * Handle wallet transaction status updates for shares
   * This replaces the original gRPC event handling
   */
  async handleWalletTransactionUpdate(
    sharesTransactionId: string,
    paymentStatus: 'COMPLETE' | 'PROCESSING' | 'FAILED' | 'PENDING',
    error?: string,
  ) {
    const startTime = Date.now();
    let success = false;
    let errorType = '';
    let sharesStatus: SharesTxStatus = SharesTxStatus.UNRECOGNIZED;
    let userId = '';
    let offerId = '';
    let quantity = 0;

    this.logger.log(
      `Processing wallet transaction update - sharesId: ${sharesTransactionId}, status: ${paymentStatus}`,
    );

    if (error) {
      this.logger.error(
        `Wallet transaction ${sharesTransactionId} failed with error: ${error}`,
      );
      errorType = 'WALLET_TX_ERROR';
    }

    try {
      const sharesTx = await this.sharesModel
        .findById(sharesTransactionId)
        .exec();
      if (!sharesTx) {
        this.logger.warn(
          `No shares transaction found with ID ${sharesTransactionId}`,
        );
        errorType = 'SHARES_TX_NOT_FOUND';
        return;
      }

      userId = sharesTx.userId;
      offerId = sharesTx.offerId;
      quantity = sharesTx.quantity;

      switch (paymentStatus) {
        case 'COMPLETE':
          sharesStatus = SharesTxStatus.COMPLETE;
          break;
        case 'PROCESSING':
          sharesStatus = SharesTxStatus.PROCESSING;
          break;
        case 'FAILED':
          sharesStatus = SharesTxStatus.FAILED;
          errorType = 'PAYMENT_FAILED';
          break;
        case 'PENDING':
        default:
          sharesStatus = sharesTx.status;
          break;
      }

      const context: ServiceContext = {
        userId: '',
        scope: PermissionScope.GLOBAL,
        permissions: [],
        user: {} as AuthenticatedUser,
      };

      await this.updateSharesInternal(context, {
        sharesId: sharesTransactionId,
        updates: { status: sharesStatus },
      });

      if (
        sharesStatus === SharesTxStatus.COMPLETE ||
        sharesStatus === SharesTxStatus.APPROVED
      ) {
        try {
          const offer = await this.sharesOfferModel
            .findById(sharesTx.offerId)
            .exec();
          if (offer) {
            await this.sharesOfferModel
              .findByIdAndUpdate(sharesTx.offerId, {
                subscribedQuantity:
                  offer.subscribedQuantity + sharesTx.quantity,
              })
              .exec();

            this.logger.log(
              `Updated offer ${sharesTx.offerId} subscribed quantity to ${
                offer.subscribedQuantity + sharesTx.quantity
              }`,
            );

            success = true;

            const allOffers = await this.viewOffers(context);
            const totalSharesAvailable = allOffers.totalOfferQuantity;

            const userShares = await this.viewShares(context, {
              userId: sharesTx.userId,
              pagination: {
                page: this.DEFAULT_PAGE,
                size: this.DEFAULT_PAGE_SIZE,
              },
            });

            const currentHoldings = userShares.shareHoldings;
            const percentageOfTotal =
              (currentHoldings / totalSharesAvailable) * 100;

            this.metricsService.recordSharesOwnershipMetric({
              userId: sharesTx.userId,
              quantity: currentHoldings,
              percentageOfTotal,
              limitReached: percentageOfTotal >= 15,
            });
          } else {
            this.logger.warn(`Offer with ID ${sharesTx.offerId} not found`);
            errorType = 'OFFER_NOT_FOUND';
          }
        } catch (updateError) {
          this.logger.error(`Error updating offer: ${updateError.message}`);
          errorType = 'OFFER_UPDATE_ERROR';
        }
      }

      this.logger.log(
        `Updated shares transaction ${sharesTransactionId} to ${SharesTxStatus[sharesStatus]} status`,
      );
    } catch (err) {
      this.logger.error(`Error processing wallet transaction: ${err.message}`);
      errorType = errorType || 'PROCESSING_ERROR';
    } finally {
      const duration = Date.now() - startTime;

      if (userId && offerId) {
        this.metricsService.recordSharesSubscriptionMetric({
          userId,
          offerId,
          quantity,
          success,
          duration,
          errorType: success ? undefined : errorType,
        });

        this.logger.log(
          `Recorded wallet transaction metrics - UserId: ${userId}, ` +
            `OfferId: ${offerId}, Quantity: ${quantity}, Status: ${SharesTxStatus[sharesStatus]}, ` +
            `Success: ${success}, Duration: ${duration}ms${errorType ? `, Error: ${errorType}` : ''}`,
        );
      }
    }
  }

  private async getPaginatedShareTx(
    query: { userId: string } | null,
    pagination: { page: number; size: number },
  ) {
    try {
      const { page, size } = pagination;
      const filter = {
        ...(query || {}),
        status: { $ne: SharesTxStatus.UNRECOGNIZED },
      };

      const allShareTx = await this.sharesModel
        .find(filter)
        .sort({ createdAt: -1 })
        .exec();

      const pages = Math.ceil(allShareTx.length / size);
      const selectPage = page > pages ? pages - 1 : page - 1; // Convert to 0-based index

      const transactions = allShareTx
        .slice(selectPage * size, (selectPage + 1) * size)
        .map((tx) => ({
          id: tx._id.toString(),
          userId: tx.userId,
          offerId: tx.offerId,
          quantity: tx.quantity,
          status: tx.status,
          transfer: tx.transfer,
          createdAt: tx.createdAt.toISOString(),
          updatedAt: tx.updatedAt.toISOString(),
        }));

      return {
        transactions,
        page: selectPage + 1, // Convert back to 1-based for response
        size,
        pages: Math.max(pages, 1),
      };
    } catch (error) {
      this.logger.error(
        `Error getting paginated shares transactions: ${error.message}`,
      );
      throw error;
    }
  }
}
