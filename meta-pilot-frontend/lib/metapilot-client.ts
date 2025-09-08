/**
 * MetaPilot Solana Client
 * High-level wrapper for easy integration with React components
 */

import { Connection, PublicKey, Transaction, Keypair, sendAndConfirmTransaction } from '@solana/web3.js';
import { 
  MetaPilotPrograms, 
  DAOVotingInstructions, 
  TokenAutomationInstructions,
  YieldOptimizerInstructions,
  TransactionSchedulerInstructions,
  AccountDerivation,
  ProposalData,
  SwapConfigData,
  YieldStrategyData,
  ScheduledTransactionData
} from './solana-programs';

export interface MetaPilotClientConfig {
  connection: Connection;
  cluster?: 'localnet' | 'devnet' | 'mainnet-beta';
  commitment?: 'processed' | 'confirmed' | 'finalized';
}

export interface CreateProposalParams {
  title: string;
  description: string;
  proposalType: 'governance' | 'treasury' | 'technical';
  proposalId: number;
}

export interface CreateSwapConfigParams {
  inputMint: PublicKey;
  outputMint: PublicKey;
  amount: number;
  priceThreshold: number;
  slippage: number;
}

export interface CreateYieldStrategyParams {
  strategyType: 'conservative' | 'balanced' | 'aggressive';
}

export interface ScheduleTransactionParams {
  targetProgram: PublicKey;
  instructionData: Buffer;
  executionTime: Date;
  maxGasPrice: number;
  txId: number;
}

export class MetaPilotClient {
  private programs: MetaPilotPrograms;
  private connection: Connection;
  private cluster: 'localnet' | 'devnet' | 'mainnet-beta';

  constructor(config: MetaPilotClientConfig) {
    this.connection = config.connection;
    this.cluster = config.cluster || 'devnet';
    this.programs = new MetaPilotPrograms(this.connection, this.cluster);
  }

  // ============== DAO VOTING METHODS ==============

  async createProposal(
    params: CreateProposalParams,
    wallet: Keypair
  ): Promise<{ signature: string; proposalPDA: PublicKey }> {
    const [proposalPDA] = await AccountDerivation.deriveProposalPDA(params.proposalId);
    
    const proposalTypeMap = {
      'governance': 0,
      'treasury': 1,
      'technical': 2,
    };

    const instruction = DAOVotingInstructions.createProposal(
      proposalPDA,
      wallet.publicKey,
      params.title,
      params.description,
      proposalTypeMap[params.proposalType]
    );

    const transaction = new Transaction().add(instruction);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );

    return { signature, proposalPDA };
  }

  async castVote(
    proposalId: number,
    support: boolean,
    wallet: Keypair
  ): Promise<string> {
    const [proposalPDA] = await AccountDerivation.deriveProposalPDA(proposalId);
    
    const instruction = DAOVotingInstructions.castVote(
      proposalPDA,
      wallet.publicKey,
      support
    );

    const transaction = new Transaction().add(instruction);
    return await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );
  }

  async getProposal(proposalId: number): Promise<ProposalData | null> {
    return await this.programs.getProposal(proposalId);
  }

  async getAllProposals(): Promise<ProposalData[]> {
    // This would require implementing a way to fetch all proposal accounts
    // For now, return empty array as implementation depends on indexing strategy
    console.warn('getAllProposals not implemented - requires account indexing');
    return [];
  }

  // ============== TOKEN AUTOMATION METHODS ==============

  async createSwapConfig(
    params: CreateSwapConfigParams,
    wallet: Keypair
  ): Promise<{ signature: string; swapConfigPDA: PublicKey }> {
    const [swapConfigPDA] = await AccountDerivation.deriveSwapConfigPDA(
      wallet.publicKey,
      params.inputMint
    );

    const instruction = TokenAutomationInstructions.createSwapConfig(
      swapConfigPDA,
      wallet.publicKey,
      params.inputMint,
      params.outputMint,
      params.amount,
      params.priceThreshold,
      params.slippage
    );

    const transaction = new Transaction().add(instruction);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );

    return { signature, swapConfigPDA };
  }

  async executeSwap(
    inputMint: PublicKey,
    inputTokenAccount: PublicKey,
    outputTokenAccount: PublicKey,
    wallet: Keypair
  ): Promise<string> {
    const [swapConfigPDA] = await AccountDerivation.deriveSwapConfigPDA(
      wallet.publicKey,
      inputMint
    );

    const instruction = TokenAutomationInstructions.executeSwap(
      swapConfigPDA,
      wallet.publicKey,
      inputTokenAccount,
      outputTokenAccount
    );

    const transaction = new Transaction().add(instruction);
    return await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );
  }

  async getSwapConfig(owner: PublicKey, inputMint: PublicKey): Promise<SwapConfigData | null> {
    return await this.programs.getSwapConfig(owner, inputMint);
  }

  // ============== YIELD OPTIMIZER METHODS ==============

  async createYieldStrategy(
    params: CreateYieldStrategyParams,
    wallet: Keypair
  ): Promise<{ signature: string; strategyPDA: PublicKey }> {
    const strategyTypeMap = {
      'conservative': 0,
      'balanced': 1,
      'aggressive': 2,
    };

    const strategyType = strategyTypeMap[params.strategyType];
    const [strategyPDA] = await AccountDerivation.deriveYieldStrategyPDA(
      wallet.publicKey,
      strategyType
    );

    const instruction = YieldOptimizerInstructions.createStrategy(
      strategyPDA,
      wallet.publicKey,
      strategyType
    );

    const transaction = new Transaction().add(instruction);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );

    return { signature, strategyPDA };
  }

  async depositToStrategy(
    strategyType: number,
    tokenAccount: PublicKey,
    amount: number,
    wallet: Keypair
  ): Promise<string> {
    const [strategyPDA] = await AccountDerivation.deriveYieldStrategyPDA(
      wallet.publicKey,
      strategyType
    );

    const instruction = YieldOptimizerInstructions.deposit(
      strategyPDA,
      wallet.publicKey,
      tokenAccount,
      amount
    );

    const transaction = new Transaction().add(instruction);
    return await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );
  }

  async getYieldStrategy(owner: PublicKey, strategyType: number): Promise<YieldStrategyData | null> {
    return await this.programs.getYieldStrategy(owner, strategyType);
  }

  // ============== TRANSACTION SCHEDULER METHODS ==============

  async scheduleTransaction(
    params: ScheduleTransactionParams,
    wallet: Keypair
  ): Promise<{ signature: string; scheduledTxPDA: PublicKey }> {
    const [scheduledTxPDA] = await AccountDerivation.deriveScheduledTxPDA(
      wallet.publicKey,
      params.txId
    );

    const instruction = TransactionSchedulerInstructions.scheduleTransaction(
      scheduledTxPDA,
      wallet.publicKey,
      params.targetProgram,
      params.instructionData,
      Math.floor(params.executionTime.getTime() / 1000), // Convert to Unix timestamp
      params.maxGasPrice
    );

    const transaction = new Transaction().add(instruction);
    const signature = await sendAndConfirmTransaction(
      this.connection,
      transaction,
      [wallet],
      { commitment: 'confirmed' }
    );

    return { signature, scheduledTxPDA };
  }

  async getScheduledTransaction(owner: PublicKey, txId: number): Promise<ScheduledTransactionData | null> {
    return await this.programs.getScheduledTransaction(owner, txId);
  }

  // ============== UTILITY METHODS ==============

  async getConnectionHealth(): Promise<{
    cluster: string;
    health: 'healthy' | 'degraded' | 'offline';
    latency: number;
  }> {
    const start = Date.now();
    try {
      await this.connection.getSlot();
      const latency = Date.now() - start;
      
      return {
        cluster: this.cluster,
        health: latency < 1000 ? 'healthy' : 'degraded',
        latency,
      };
    } catch (error) {
      return {
        cluster: this.cluster,
        health: 'offline',
        latency: -1,
      };
    }
  }

  async estimateTransactionFee(transaction: Transaction): Promise<number> {
    try {
      const fee = await this.connection.getFeeForMessage(
        transaction.compileMessage(),
        'confirmed'
      );
      return fee?.value || 0;
    } catch (error) {
      console.error('Error estimating transaction fee:', error);
      return 5000; // Default fallback (5000 lamports)
    }
  }

  // Get all user accounts for dashboard display
  async getUserAccounts(userPublicKey: PublicKey) {
    try {
      // In a real implementation, you'd need to fetch all PDA accounts
      // for the user across all programs. This requires account indexing.
      
      const accounts = {
        proposals: [] as ProposalData[],
        swapConfigs: [] as SwapConfigData[],
        yieldStrategies: [] as YieldStrategyData[],
        scheduledTransactions: [] as ScheduledTransactionData[],
      };

      // This is a simplified version - in production you'd use:
      // 1. getProgramAccounts with proper filters
      // 2. Account indexing service
      // 3. Cached account data

      console.warn('getUserAccounts requires account indexing implementation');
      return accounts;
    } catch (error) {
      console.error('Error fetching user accounts:', error);
      throw error;
    }
  }
}

// Helper function to create client with common configurations
export function createMetaPilotClient(
  rpcUrl: string,
  cluster: 'localnet' | 'devnet' | 'mainnet-beta' = 'devnet'
): MetaPilotClient {
  const connection = new Connection(rpcUrl, 'confirmed');
  return new MetaPilotClient({ connection, cluster });
}

// Types are already exported above, no need to re-export
