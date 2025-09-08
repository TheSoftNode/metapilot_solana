/**
 * MetaPilot AI + Solana Integration
 * Connects AI Engine decisions with Solana smart contract execution
 */

import { PublicKey, Keypair } from '@solana/web3.js';
// AI Engine will be imported from the meta-pilot-ai-engine when integrated
// For now, we'll use a simplified interface
import { MetaPilotClient, CreateProposalParams } from '../lib/metapilot-client';

// Simplified AI Engine interface for integration
interface AIEngine {
  analyze(
    type: string,
    input: any,
    context?: any
  ): Promise<{
    success: boolean;
    decision?: AIDecision;
    error?: string;
  }>;
}

export interface AIDecision {
  action: 'EXECUTE' | 'WAIT' | 'REJECT';
  confidence: number;
  reasoning: string[];
  metadata: {
    vote?: 'FOR' | 'AGAINST';
    swapTiming?: 'NOW' | 'LATER';
    gasOptimization?: {
      suggestedGasPrice: number;
      optimalTiming: Date;
      estimatedSavings: number;
    };
    allocations?: number[];
    [key: string]: any;
  };
}

export interface AutomationRule {
  id: string;
  name: string;
  type: 'dao_voting' | 'token_swap' | 'yield_optimization' | 'gas_optimization';
  enabled: boolean;
  conditions: any;
  aiPrompt: string;
}

export class MetaPilotAIIntegration {
  constructor(
    private aiEngine: AIEngine,
    private solanaClient: MetaPilotClient,
    private userWallet: Keypair
  ) {}

  // ============== DAO VOTING AUTOMATION ==============

  async analyzeAndVoteOnProposal(
    proposalId: number,
    userRules: AutomationRule[],
    proposalContext?: any
  ): Promise<{
    aiDecision: AIDecision;
    executed: boolean;
    signature?: string;
    error?: string;
  }> {
    try {
      // 1. Fetch proposal data from blockchain
      const proposal = await this.solanaClient.getProposal(proposalId);
      if (!proposal) {
        throw new Error('Proposal not found');
      }

      // 2. Prepare context for AI analysis
      const analysisContext = {
        blockchain: 'solana',
        proposal: {
          id: proposalId,
          title: proposal.title,
          description: proposal.description,
          proposalType: proposal.proposalType,
          votesFor: proposal.votesFor,
          votesAgainst: proposal.votesAgainst,
          deadline: proposal.deadline,
        },
        userRules: userRules.filter(rule => rule.type === 'dao_voting' && rule.enabled),
        ...proposalContext,
      };

      // 3. Get AI decision
      const analysis = await this.aiEngine.analyze('proposal', {
        proposalId: proposalId.toString(),
        daoName: 'MetaPilot DAO', // Could be dynamic
        proposalText: proposal.description,
        proposalType: this.mapProposalType(proposal.proposalType),
      }, analysisContext);

      if (!analysis.success) {
        return {
          aiDecision: {
            action: 'REJECT',
            confidence: 0,
            reasoning: ['AI analysis failed'],
            metadata: {},
          },
          executed: false,
          error: 'AI analysis failed',
        };
      }

      const aiDecision = analysis.decision as AIDecision;

      // 4. Execute based on AI decision
      if (aiDecision.action === 'EXECUTE' && aiDecision.metadata.vote) {
        try {
          const signature = await this.solanaClient.castVote(
            proposalId,
            aiDecision.metadata.vote === 'FOR',
            this.userWallet
          );

          return {
            aiDecision,
            executed: true,
            signature,
          };
        } catch (executionError) {
          return {
            aiDecision,
            executed: false,
            error: executionError instanceof Error ? executionError.message : 'Execution failed',
          };
        }
      }

      return {
        aiDecision,
        executed: false,
      };

    } catch (error) {
      return {
        aiDecision: {
          action: 'REJECT',
          confidence: 0,
          reasoning: ['Analysis error: ' + (error instanceof Error ? error.message : 'Unknown error')],
          metadata: {},
        },
        executed: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // ============== TOKEN SWAP OPTIMIZATION ==============

  async analyzeSwapTiming(
    inputMint: PublicKey,
    outputMint: PublicKey,
    amount: number,
    userRules: AutomationRule[]
  ): Promise<{
    aiDecision: AIDecision;
    shouldExecute: boolean;
    optimalTiming?: Date;
  }> {
    try {
      // 1. Get current market conditions (would integrate with price APIs)
      const marketContext = {
        inputToken: inputMint.toString(),
        outputToken: outputMint.toString(),
        amount,
        // Add real market data here
        marketConditions: await this.getMarketConditions(inputMint, outputMint),
      };

      // 2. Analyze with AI
      const analysis = await this.aiEngine.analyze('transaction', {
        transactionType: 'swap',
        fromToken: inputMint.toString(),
        toToken: outputMint.toString(),
        amount: amount.toString(),
      }, {
        blockchain: 'solana',
        userRules: userRules.filter(rule => rule.type === 'token_swap' && rule.enabled),
        ...marketContext,
      });

      if (!analysis.success) {
        return {
          aiDecision: {
            action: 'WAIT',
            confidence: 0,
            reasoning: ['Market analysis unavailable'],
            metadata: {},
          },
          shouldExecute: false,
        };
      }

      const aiDecision = analysis.decision as AIDecision;
      
      return {
        aiDecision,
        shouldExecute: aiDecision.action === 'EXECUTE',
        optimalTiming: aiDecision.metadata.gasOptimization?.optimalTiming,
      };

    } catch (error) {
      return {
        aiDecision: {
          action: 'WAIT',
          confidence: 0,
          reasoning: ['Error analyzing swap timing'],
          metadata: {},
        },
        shouldExecute: false,
      };
    }
  }

  // ============== YIELD OPTIMIZATION ==============

  async optimizeYieldStrategy(
    currentStrategy: any,
    userRules: AutomationRule[]
  ): Promise<{
    aiDecision: AIDecision;
    recommendedAllocations?: number[];
    shouldRebalance: boolean;
  }> {
    try {
      const analysis = await this.aiEngine.analyze('market', {
        portfolioType: 'yield_optimization',
        currentAllocations: currentStrategy.allocations,
        totalValue: currentStrategy.totalDeposited,
        currentYield: currentStrategy.currentYield,
      }, {
        blockchain: 'solana',
        userRules: userRules.filter(rule => rule.type === 'yield_optimization' && rule.enabled),
        marketConditions: await this.getYieldMarketConditions(),
      });

      if (!analysis.success) {
        return {
          aiDecision: {
            action: 'WAIT',
            confidence: 0,
            reasoning: ['Unable to analyze yield opportunities'],
            metadata: {},
          },
          shouldRebalance: false,
        };
      }

      const aiDecision = analysis.decision as AIDecision;

      return {
        aiDecision,
        recommendedAllocations: aiDecision.metadata.allocations,
        shouldRebalance: aiDecision.action === 'EXECUTE',
      };

    } catch (error) {
      return {
        aiDecision: {
          action: 'WAIT',
          confidence: 0,
          reasoning: ['Error optimizing yield strategy'],
          metadata: {},
        },
        shouldRebalance: false,
      };
    }
  }

  // ============== GAS OPTIMIZATION ==============

  async optimizeTransactionTiming(
    transactionType: string,
    priority: 'low' | 'medium' | 'high' = 'medium'
  ): Promise<{
    aiDecision: AIDecision;
    optimalGasPrice: number;
    optimalTiming: Date;
    estimatedSavings: number;
  }> {
    try {
      const networkConditions = await this.getNetworkConditions();
      
      const analysis = await this.aiEngine.analyze('transaction', {
        transactionType,
        priority,
        networkConditions,
      }, {
        blockchain: 'solana',
        optimization: 'gas',
      });

      if (!analysis.success) {
        return {
          aiDecision: {
            action: 'EXECUTE',
            confidence: 50,
            reasoning: ['Using default gas settings'],
            metadata: {},
          },
          optimalGasPrice: 5000, // Default lamports
          optimalTiming: new Date(),
          estimatedSavings: 0,
        };
      }

      const aiDecision = analysis.decision as AIDecision;
      const gasOptimization = aiDecision.metadata.gasOptimization;

      return {
        aiDecision,
        optimalGasPrice: gasOptimization?.suggestedGasPrice || 5000,
        optimalTiming: gasOptimization?.optimalTiming || new Date(),
        estimatedSavings: gasOptimization?.estimatedSavings || 0,
      };

    } catch (error) {
      return {
        aiDecision: {
          action: 'EXECUTE',
          confidence: 0,
          reasoning: ['Error optimizing gas, using defaults'],
          metadata: {},
        },
        optimalGasPrice: 5000,
        optimalTiming: new Date(),
        estimatedSavings: 0,
      };
    }
  }

  // ============== HELPER METHODS ==============

  private mapProposalType(proposalType: number): string {
    const types = ['governance', 'treasury', 'technical'];
    return types[proposalType] || 'governance';
  }

  private async getMarketConditions(inputMint: PublicKey, outputMint: PublicKey) {
    // In a real implementation, this would fetch from price APIs like Jupiter, CoinGecko, etc.
    return {
      inputPrice: 1.0, // Placeholder
      outputPrice: 1.0, // Placeholder
      volume24h: 1000000, // Placeholder
      priceChange24h: 0.05, // Placeholder
      liquidity: 5000000, // Placeholder
    };
  }

  private async getYieldMarketConditions() {
    // Fetch current yield rates from various Solana DeFi protocols
    return {
      marinade: { apy: 0.065 }, // Marinade staking
      solend: { apy: 0.045 }, // Solend lending
      mango: { apy: 0.075 }, // Mango markets
      raydium: { apy: 0.055 }, // Raydium LP
    };
  }

  private async getNetworkConditions() {
    // Get current Solana network conditions
    try {
      const recentPerformance = await this.solanaClient['connection'].getRecentPerformanceSamples(1);
      const slot = await this.solanaClient['connection'].getSlot();
      
      return {
        currentSlot: slot,
        averageSlotTime: recentPerformance[0]?.samplePeriodSecs || 0.4,
        congestion: 'low', // Would calculate based on recent block times
        averageFee: 5000, // Would calculate from recent transactions
      };
    } catch (error) {
      return {
        currentSlot: 0,
        averageSlotTime: 0.4,
        congestion: 'unknown',
        averageFee: 5000,
      };
    }
  }
}

// Factory function for easy initialization
export function createAIIntegration(
  aiEngine: AIEngine,
  solanaClient: MetaPilotClient,
  userWallet: Keypair
): MetaPilotAIIntegration {
  return new MetaPilotAIIntegration(aiEngine, solanaClient, userWallet);
}
