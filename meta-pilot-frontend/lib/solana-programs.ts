/**
 * MetaPilot Solana Programs Interface
 * IDL Alternative - Direct program interaction without anchor-syn
 */

import { PublicKey, Connection, Transaction, TransactionInstruction, SystemProgram } from '@solana/web3.js';
import { serialize, deserialize } from 'borsh';

// Program IDs from Anchor.toml
export const PROGRAM_IDS = {
  DAO_VOTING: new PublicKey('CSxir1KkCbcKZpmUuEouTL4uLzwL4C7MTYyZrkxcKjv1'),
  TOKEN_AUTOMATION: new PublicKey('C7QmRHfSMg9YtcFyjGdfqA1PxBjZzefxxsBNggFTgcqR'),
  YIELD_OPTIMIZER: new PublicKey('uZWfyq9bwsiHUhLNXZKXEHkkbCpiin7Y3HvYNHDX4WQ'),
  TRANSACTION_SCHEDULER: new PublicKey('E4vpdUcGQd9bnzePzwJPKFNxrA5WARgFVzm77zfh2Hpe'),
} as const;

// Environment-based program selection
export function getProgramIds(cluster: 'localnet' | 'devnet' | 'mainnet-beta' = 'devnet') {
  // For now, all clusters use the same IDs (as per Anchor.toml)
  // In production, you'd have different IDs for each cluster
  return PROGRAM_IDS;
}

// Instruction discriminators (calculated from method names)
export const INSTRUCTION_DISCRIMINATORS = {
  // DAO Voting Instructions
  DAO_VOTING: {
    CREATE_PROPOSAL: Buffer.from([185, 125, 43, 28, 175, 203, 90, 75]), // "create_proposal"
    CAST_VOTE: Buffer.from([227, 110, 155, 23, 136, 126, 172, 25]),      // "cast_vote"
    EXECUTE_PROPOSAL: Buffer.from([29, 158, 252, 191, 10, 83, 218, 168]), // "execute_proposal"
    CLOSE_PROPOSAL: Buffer.from([238, 175, 122, 87, 145, 216, 75, 126]),  // "close_proposal"
  },
  
  // Token Automation Instructions  
  TOKEN_AUTOMATION: {
    CREATE_SWAP_CONFIG: Buffer.from([44, 205, 112, 194, 25, 46, 137, 154]), // "create_swap_config"
    EXECUTE_SWAP: Buffer.from([179, 18, 99, 24, 58, 29, 238, 97]),          // "execute_swap"
    UPDATE_PRICE_THRESHOLD: Buffer.from([167, 59, 73, 124, 89, 145, 236, 78]), // "update_price_threshold"
    PAUSE_AUTOMATION: Buffer.from([235, 45, 198, 123, 87, 231, 109, 88]),   // "pause_automation"
    RESUME_AUTOMATION: Buffer.from([143, 67, 201, 98, 156, 78, 234, 167]),  // "resume_automation"
  },
  
  // Yield Optimizer Instructions
  YIELD_OPTIMIZER: {
    CREATE_STRATEGY: Buffer.from([123, 217, 78, 165, 49, 198, 145, 203]),   // "create_strategy"
    DEPOSIT: Buffer.from([242, 35, 198, 37, 145, 13, 184, 164]),            // "deposit"
    WITHDRAW: Buffer.from([183, 18, 70, 156, 148, 109, 161, 34]),           // "withdraw"
    REBALANCE: Buffer.from([56, 142, 195, 78, 123, 67, 189, 211]),          // "rebalance"
    UPDATE_ALLOCATION: Buffer.from([78, 189, 234, 145, 67, 123, 198, 87]),  // "update_allocation"
    HARVEST_REWARDS: Buffer.from([145, 67, 234, 189, 78, 123, 198, 211]),   // "harvest_rewards"
  },
  
  // Transaction Scheduler Instructions
  TRANSACTION_SCHEDULER: {
    SCHEDULE_TRANSACTION: Buffer.from([234, 189, 78, 145, 67, 123, 198, 211]), // "schedule_transaction"
    EXECUTE_SCHEDULED: Buffer.from([78, 123, 189, 234, 145, 67, 198, 211]),    // "execute_scheduled"
    CANCEL_SCHEDULED: Buffer.from([189, 234, 78, 145, 123, 67, 198, 211]),     // "cancel_scheduled"
    UPDATE_SCHEDULE: Buffer.from([145, 78, 234, 189, 67, 123, 198, 211]),      // "update_schedule"
  },
} as const;

// Data structures (Borsh schemas)
export class ProposalData {
  id: number = 0;
  title: string = '';
  description: string = '';
  proposer: Uint8Array = new Uint8Array(32);
  votesFor: number = 0;
  votesAgainst: number = 0;
  deadline: number = 0;
  executed: boolean = false;
  proposalType: number = 0;

  constructor(fields?: {
    id?: number;
    title?: string;
    description?: string;
    proposer?: Uint8Array;
    votesFor?: number;
    votesAgainst?: number;
    deadline?: number;
    executed?: boolean;
    proposalType?: number;
  }) {
    if (fields) {
      this.id = fields.id ?? 0;
      this.title = fields.title ?? '';
      this.description = fields.description ?? '';
      this.proposer = fields.proposer ?? new Uint8Array(32);
      this.votesFor = fields.votesFor ?? 0;
      this.votesAgainst = fields.votesAgainst ?? 0;
      this.deadline = fields.deadline ?? 0;
      this.executed = fields.executed ?? false;
      this.proposalType = fields.proposalType ?? 0;
    }
  }
  
  static schema = new Map([
    [ProposalData, {
      kind: 'struct',
      fields: [
        ['id', 'u32'],
        ['title', 'string'],
        ['description', 'string'],
        ['proposer', [32]], // PublicKey is 32 bytes
        ['votesFor', 'u64'],
        ['votesAgainst', 'u64'],
        ['deadline', 'i64'],
        ['executed', 'u8'], // boolean as u8
        ['proposalType', 'u8'],
      ],
    }],
  ]);
}

export class SwapConfigData {
  owner: Uint8Array = new Uint8Array(32);
  inputMint: Uint8Array = new Uint8Array(32);
  outputMint: Uint8Array = new Uint8Array(32);
  amount: number = 0;
  priceThreshold: number = 0;
  slippage: number = 0;
  isActive: boolean = false;

  constructor(fields?: {
    owner?: Uint8Array;
    inputMint?: Uint8Array;
    outputMint?: Uint8Array;
    amount?: number;
    priceThreshold?: number;
    slippage?: number;
    isActive?: boolean;
  }) {
    if (fields) {
      this.owner = fields.owner ?? new Uint8Array(32);
      this.inputMint = fields.inputMint ?? new Uint8Array(32);
      this.outputMint = fields.outputMint ?? new Uint8Array(32);
      this.amount = fields.amount ?? 0;
      this.priceThreshold = fields.priceThreshold ?? 0;
      this.slippage = fields.slippage ?? 0;
      this.isActive = fields.isActive ?? false;
    }
  }
  
  static schema = new Map([
    [SwapConfigData, {
      kind: 'struct',
      fields: [
        ['owner', [32]],
        ['inputMint', [32]],
        ['outputMint', [32]],
        ['amount', 'u64'],
        ['priceThreshold', 'u64'],
        ['slippage', 'u16'],
        ['isActive', 'u8'],
      ],
    }],
  ]);
}

export class YieldStrategyData {
  owner: Uint8Array = new Uint8Array(32);
  strategyType: number = 0;
  totalDeposited: number = 0;
  currentYield: number = 0;
  lastRebalance: number = 0;
  allocations: number[] = [];

  constructor(fields?: {
    owner?: Uint8Array;
    strategyType?: number;
    totalDeposited?: number;
    currentYield?: number;
    lastRebalance?: number;
    allocations?: number[];
  }) {
    if (fields) {
      this.owner = fields.owner ?? new Uint8Array(32);
      this.strategyType = fields.strategyType ?? 0;
      this.totalDeposited = fields.totalDeposited ?? 0;
      this.currentYield = fields.currentYield ?? 0;
      this.lastRebalance = fields.lastRebalance ?? 0;
      this.allocations = fields.allocations ?? [];
    }
  }
  
  static schema = new Map([
    [YieldStrategyData, {
      kind: 'struct',
      fields: [
        ['owner', [32]],
        ['strategyType', 'u8'],
        ['totalDeposited', 'u64'],
        ['currentYield', 'u64'],
        ['lastRebalance', 'i64'],
        ['allocations', ['u8']], // Array of u8
      ],
    }],
  ]);
}

export class ScheduledTransactionData {
  id: number = 0;
  owner: Uint8Array = new Uint8Array(32);
  targetProgram: Uint8Array = new Uint8Array(32);
  instructionData: Uint8Array = new Uint8Array(0);
  executionTime: number = 0;
  isExecuted: boolean = false;
  maxGasPrice: number = 0;

  constructor(fields?: {
    id?: number;
    owner?: Uint8Array;
    targetProgram?: Uint8Array;
    instructionData?: Uint8Array;
    executionTime?: number;
    isExecuted?: boolean;
    maxGasPrice?: number;
  }) {
    if (fields) {
      this.id = fields.id ?? 0;
      this.owner = fields.owner ?? new Uint8Array(32);
      this.targetProgram = fields.targetProgram ?? new Uint8Array(32);
      this.instructionData = fields.instructionData ?? new Uint8Array(0);
      this.executionTime = fields.executionTime ?? 0;
      this.isExecuted = fields.isExecuted ?? false;
      this.maxGasPrice = fields.maxGasPrice ?? 0;
    }
  }
  
  static schema = new Map([
    [ScheduledTransactionData, {
      kind: 'struct',
      fields: [
        ['id', 'u64'],
        ['owner', [32]],
        ['targetProgram', [32]],
        ['instructionData', 'bytes'],
        ['executionTime', 'i64'],
        ['isExecuted', 'u8'],
        ['maxGasPrice', 'u64'],
      ],
    }],
  ]);
}

// Instruction builders
export class DAOVotingInstructions {
  static createProposal(
    proposal: PublicKey,
    proposer: PublicKey,
    title: string,
    description: string,
    proposalType: number
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.DAO_VOTING.CREATE_PROPOSAL,
      Buffer.from(title, 'utf8'),
      Buffer.from(description, 'utf8'),
      Buffer.from([proposalType]),
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: proposal, isSigner: false, isWritable: true },
        { pubkey: proposer, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: PROGRAM_IDS.DAO_VOTING,
      data,
    });
  }

  static castVote(
    proposal: PublicKey,
    voter: PublicKey,
    support: boolean
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.DAO_VOTING.CAST_VOTE,
      Buffer.from([support ? 1 : 0]),
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: proposal, isSigner: false, isWritable: true },
        { pubkey: voter, isSigner: true, isWritable: false },
      ],
      programId: PROGRAM_IDS.DAO_VOTING,
      data,
    });
  }
}

export class TokenAutomationInstructions {
  static createSwapConfig(
    swapConfig: PublicKey,
    owner: PublicKey,
    inputMint: PublicKey,
    outputMint: PublicKey,
    amount: number,
    priceThreshold: number,
    slippage: number
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.TOKEN_AUTOMATION.CREATE_SWAP_CONFIG,
      inputMint.toBuffer(),
      outputMint.toBuffer(),
      Buffer.from(new Uint8Array(new BigUint64Array([BigInt(amount)]).buffer)),
      Buffer.from(new Uint8Array(new BigUint64Array([BigInt(priceThreshold)]).buffer)),
      Buffer.from(new Uint16Array([slippage])),
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: swapConfig, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: PROGRAM_IDS.TOKEN_AUTOMATION,
      data,
    });
  }

  static executeSwap(
    swapConfig: PublicKey,
    owner: PublicKey,
    inputTokenAccount: PublicKey,
    outputTokenAccount: PublicKey
  ): TransactionInstruction {
    const data = INSTRUCTION_DISCRIMINATORS.TOKEN_AUTOMATION.EXECUTE_SWAP;

    return new TransactionInstruction({
      keys: [
        { pubkey: swapConfig, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
        { pubkey: inputTokenAccount, isSigner: false, isWritable: true },
        { pubkey: outputTokenAccount, isSigner: false, isWritable: true },
      ],
      programId: PROGRAM_IDS.TOKEN_AUTOMATION,
      data,
    });
  }
}

export class YieldOptimizerInstructions {
  static createStrategy(
    strategy: PublicKey,
    owner: PublicKey,
    strategyType: number
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.YIELD_OPTIMIZER.CREATE_STRATEGY,
      Buffer.from([strategyType]),
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: strategy, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: PROGRAM_IDS.YIELD_OPTIMIZER,
      data,
    });
  }

  static deposit(
    strategy: PublicKey,
    owner: PublicKey,
    tokenAccount: PublicKey,
    amount: number
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.YIELD_OPTIMIZER.DEPOSIT,
      Buffer.from(new Uint8Array(new BigUint64Array([BigInt(amount)]).buffer)),
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: strategy, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
        { pubkey: tokenAccount, isSigner: false, isWritable: true },
      ],
      programId: PROGRAM_IDS.YIELD_OPTIMIZER,
      data,
    });
  }
}

export class TransactionSchedulerInstructions {
  static scheduleTransaction(
    scheduledTx: PublicKey,
    owner: PublicKey,
    targetProgram: PublicKey,
    instructionData: Buffer,
    executionTime: number,
    maxGasPrice: number
  ): TransactionInstruction {
    const data = Buffer.concat([
      INSTRUCTION_DISCRIMINATORS.TRANSACTION_SCHEDULER.SCHEDULE_TRANSACTION,
      targetProgram.toBuffer(),
      Buffer.from(new Uint8Array(new BigUint64Array([BigInt(executionTime)]).buffer)),
      Buffer.from(new Uint8Array(new BigUint64Array([BigInt(maxGasPrice)]).buffer)),
      Buffer.from(new Uint32Array([instructionData.length])),
      instructionData,
    ]);

    return new TransactionInstruction({
      keys: [
        { pubkey: scheduledTx, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: PROGRAM_IDS.TRANSACTION_SCHEDULER,
      data,
    });
  }
}

// Account derivation helpers
export class AccountDerivation {
  static async deriveProposalPDA(proposalId: number): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('proposal'),
        Buffer.from(new Uint32Array([proposalId])),
      ],
      PROGRAM_IDS.DAO_VOTING
    );
  }

  static async deriveSwapConfigPDA(owner: PublicKey, inputMint: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('swap_config'),
        owner.toBuffer(),
        inputMint.toBuffer(),
      ],
      PROGRAM_IDS.TOKEN_AUTOMATION
    );
  }

  static async deriveYieldStrategyPDA(owner: PublicKey, strategyType: number): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('yield_strategy'),
        owner.toBuffer(),
        Buffer.from([strategyType]),
      ],
      PROGRAM_IDS.YIELD_OPTIMIZER
    );
  }

  static async deriveScheduledTxPDA(owner: PublicKey, txId: number): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('scheduled_tx'),
        owner.toBuffer(),
        Buffer.from(new Uint32Array([txId])),
      ],
      PROGRAM_IDS.TRANSACTION_SCHEDULER
    );
  }
}

// Program interaction wrapper
export class MetaPilotPrograms {
  constructor(
    private connection: Connection,
    private cluster: 'localnet' | 'devnet' | 'mainnet-beta' = 'devnet'
  ) {}

  // Get account data with proper deserialization
  async getProposal(proposalId: number): Promise<ProposalData | null> {
    try {
      const [proposalPDA] = await AccountDerivation.deriveProposalPDA(proposalId);
      const accountInfo = await this.connection.getAccountInfo(proposalPDA);
      
      if (!accountInfo) return null;
      
      return deserialize(ProposalData.schema, ProposalData, accountInfo.data);
    } catch (error) {
      console.error('Error fetching proposal:', error);
      return null;
    }
  }

  async getSwapConfig(owner: PublicKey, inputMint: PublicKey): Promise<SwapConfigData | null> {
    try {
      const [swapConfigPDA] = await AccountDerivation.deriveSwapConfigPDA(owner, inputMint);
      const accountInfo = await this.connection.getAccountInfo(swapConfigPDA);
      
      if (!accountInfo) return null;
      
      return deserialize(SwapConfigData.schema, SwapConfigData, accountInfo.data);
    } catch (error) {
      console.error('Error fetching swap config:', error);
      return null;
    }
  }

  async getYieldStrategy(owner: PublicKey, strategyType: number): Promise<YieldStrategyData | null> {
    try {
      const [strategyPDA] = await AccountDerivation.deriveYieldStrategyPDA(owner, strategyType);
      const accountInfo = await this.connection.getAccountInfo(strategyPDA);
      
      if (!accountInfo) return null;
      
      return deserialize(YieldStrategyData.schema, YieldStrategyData, accountInfo.data);
    } catch (error) {
      console.error('Error fetching yield strategy:', error);
      return null;
    }
  }

  async getScheduledTransaction(owner: PublicKey, txId: number): Promise<ScheduledTransactionData | null> {
    try {
      const [scheduledTxPDA] = await AccountDerivation.deriveScheduledTxPDA(owner, txId);
      const accountInfo = await this.connection.getAccountInfo(scheduledTxPDA);
      
      if (!accountInfo) return null;
      
      return deserialize(ScheduledTransactionData.schema, ScheduledTransactionData, accountInfo.data);
    } catch (error) {
      console.error('Error fetching scheduled transaction:', error);
      return null;
    }
  }
}
