import { Injectable, Logger, BadRequestException } from '@nestjs/common';
import {
  SorobanRpc,
  Transaction,
  FeeBumpTransaction,
  TransactionBuilder,
  Keypair,
  Networks,
  xdr,
  StrKey,
} from '@stellar/stellar-sdk';
import { ConfigService } from '../config/config.service';

@Injectable()
export class RelayService {
  private readonly logger = new Logger(RelayService.name);
  private readonly hotWallet: Keypair;

  constructor(
    private readonly configService: ConfigService,
    private readonly rpcServer: SorobanRpc.Server,
  ) {
    const secret = process.env.RELAY_HOT_WALLET_SECRET;
    if (!secret) {
      this.logger.warn('RELAY_HOT_WALLET_SECRET not set. Relay will not work.');
      // For now, don't throw to avoid crashing the app if not used
    } else {
      this.hotWallet = Keypair.fromSecret(secret);
      this.logger.log(
        `Relay active with sponsor address: ${this.hotWallet.publicKey()}`,
      );
    }
  }

  private xlmPriceCache: { usd: number; expiresAt: number } | null = null;

  private async getXlmUsdPrice(): Promise<number> {
    if (this.xlmPriceCache && Date.now() < this.xlmPriceCache.expiresAt) {
      return this.xlmPriceCache.usd;
    }
    try {
      const res = await fetch(
        'https://api.coingecko.com/api/v3/simple/price?ids=stellar&vs_currencies=usd',
      );
      const json = (await res.json()) as Record<string, Record<string, number>>;
      const usd = json?.stellar?.usd ?? 0;
      this.xlmPriceCache = { usd, expiresAt: Date.now() + 60_000 };
      return usd;
    } catch {
      return this.xlmPriceCache?.usd ?? 0;
    }
  }

  async estimateFee(xdrString: string): Promise<{
    estimatedGasXLM: string;
    estimatedGasUSD: string;
    resourceCost: unknown;
    sponsored: boolean;
  }> {
    const networkPassphrase =
      process.env.NETWORK_PASSPHRASE || Networks.TESTNET;
    let tx: Transaction | FeeBumpTransaction;
    try {
      tx = TransactionBuilder.fromXDR(xdrString, networkPassphrase);
    } catch {
      throw new BadRequestException('Invalid XDR');
    }

    const innerTx = tx instanceof FeeBumpTransaction ? tx.innerTransaction : tx;

    let resourceCost: unknown = null;
    let feeStroops = BigInt(innerTx.fee);

    try {
      const simResult = await this.rpcServer.simulateTransaction(innerTx);
      if (SorobanRpc.Api.isSimulationSuccess(simResult)) {
        const minFee = simResult.minResourceFee;
        if (minFee) feeStroops = BigInt(minFee);
        resourceCost = simResult.cost ?? null;
      }
    } catch {
      // use tx fee as fallback
    }

    const feeXlm = (Number(feeStroops) / 1e7).toFixed(7);
    const usdPrice = await this.getXlmUsdPrice();
    const feeUsd = (parseFloat(feeXlm) * usdPrice).toFixed(6);
    const sponsored = !!this.hotWallet;

    return {
      estimatedGasXLM: feeXlm,
      estimatedGasUSD: feeUsd,
      resourceCost,
      sponsored,
    };
  }

  async sponsorAndSubmit(xdrString: string): Promise<{ hash: string }> {
    if (!this.hotWallet) {
      throw new BadRequestException(
        'Relay not configured (missing secret key)',
      );
    }

    const networkPassphrase =
      process.env.NETWORK_PASSPHRASE || Networks.TESTNET;
    let tx: Transaction | FeeBumpTransaction;

    try {
      tx = TransactionBuilder.fromXDR(xdrString, networkPassphrase);
    } catch {
      throw new BadRequestException('Invalid XDR');
    }

    // Determine if it's already a fee-bump or a regular transaction
    const innerTx = tx instanceof FeeBumpTransaction ? tx.innerTransaction : tx;

    // Validate inner transaction
    await this.validateTransaction(innerTx);

    // Security: Ensure inner transaction has at least one signature from the user
    if (!innerTx.signatures || innerTx.signatures.length === 0) {
      throw new BadRequestException(
        'Inner transaction must be signed by the user',
      );
    }

    // Security: Check transaction expiration (timebounds)
    if (innerTx.timeBounds) {
      const now = Math.floor(Date.now() / 1000);
      const { minTime, maxTime } = innerTx.timeBounds;
      if (maxTime !== '0' && now > parseInt(maxTime)) {
        throw new BadRequestException('Transaction has expired');
      }
      if (minTime !== '0' && now < parseInt(minTime)) {
        throw new BadRequestException('Transaction is not yet valid');
      }
    }

    let finalTx: FeeBumpTransaction;
    if (tx instanceof FeeBumpTransaction) {
      // If it's already a fee-bump, we check if we are the intended sponsor
      if (tx.feeSource !== this.hotWallet.publicKey()) {
        throw new BadRequestException(
          `Fee source mismatch. Expected ${this.hotWallet.publicKey()}`,
        );
      }
      finalTx = tx;
    } else {
      // Create a fee-bump transaction
      // The outer fee must be at least inner_fee + base_fee.
      // We add a small margin (500 stroops) to ensure acceptance.
      const innerFee = BigInt(innerTx.fee);
      const outerFee = (innerFee + 500n).toString();

      finalTx = TransactionBuilder.buildFeeBumpTransaction(
        this.hotWallet,
        outerFee,
        innerTx,
        networkPassphrase,
      );
    }

    // Sign with relayer hot wallet
    finalTx.sign(this.hotWallet);

    // Submit back to Stellar
    try {
      const response = await this.rpcServer.sendTransaction(finalTx);

      if (response.status === 'ERROR') {
        const responseAny = response as Record<string, unknown>;
        const errorMsg =
          (responseAny.errorResultXdr as string | undefined) ||
          (response.errorResult
            ? JSON.stringify(response.errorResult)
            : 'Unknown error');
        this.logger.error(`Transaction failed: ${errorMsg}`);
        throw new BadRequestException(
          `Transaction submission failed: ${errorMsg}`,
        );
      }

      this.logger.log(`Relayed transaction ${response.hash} for contract call`);
      return { hash: response.hash };
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error';
      this.logger.error(`Relay submission error: ${msg}`);
      throw new BadRequestException(`Submission failed: ${msg}`);
    }
  }

  private async validateTransaction(tx: Transaction): Promise<void> {
    const settings = await this.configService.getSettings();
    const allowedContractId = settings.contractId;

    if (!allowedContractId) {
      this.logger.error(
        'No allowed contract ID configured in platform settings',
      );
      throw new BadRequestException('Relay target contract not configured');
    }

    // Check all operations
    if (tx.operations.length === 0) {
      throw new BadRequestException('Transaction has no operations');
    }

    for (const op of tx.operations) {
      // For Soroban, we only allow host function invocations in the relay
      if (op.type !== 'invokeHostFunction') {
        throw new BadRequestException(
          `Operation type ${op.type} not allowed in relay. Only invokeHostFunction is permitted.`,
        );
      }

      // Cast to the specific operation type to access Soroban-specific fields
      const hostFunctionOp = op as unknown as {
        func: xdr.HostFunction;
        type: string;
      };
      const hostFunction = hostFunctionOp.func;

      if (!hostFunction) {
        throw new BadRequestException(
          'Malformed host function operation: missing function definition',
        );
      }

      // Ensure it's a contract invocation
      if (
        hostFunction.switch().value !==
        xdr.HostFunctionType.hostFunctionTypeInvokeContract().value
      ) {
        throw new BadRequestException(
          'Only contract invocations are allowed in this relay',
        );
      }

      const invokeContractArgs = hostFunction.invokeContract();
      const contractAddress = invokeContractArgs.contractAddress();

      let contractId: string;
      if (
        contractAddress.switch().value ===
        xdr.ScAddressType.scAddressTypeContract().value
      ) {
        contractId = StrKey.encodeContract(contractAddress.contractId());
      } else {
        throw new BadRequestException(
          'Invalid contract address type: must be a contract ID',
        );
      }

      if (contractId !== allowedContractId) {
        this.logger.warn(
          `Unauthorized relay attempt to contract: ${contractId}`,
        );
        throw new BadRequestException(
          `Transaction directed at unauthorized contract: ${contractId}. Only ${allowedContractId} is allowed.`,
        );
      }
    }
  }
}
