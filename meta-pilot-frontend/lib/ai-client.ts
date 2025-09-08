"use client";

/**
 * MetaPilot AI Client - Frontend integration with AI Engine
 * Connects the frontend to the AI engine for automation decisions
 */

// AI Engine types
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
  userPreferences: {
    riskTolerance: 'low' | 'medium' | 'high';
    preferredVoteThreshold: number;
    maxGasPrice?: number;
    minConfidence: number;
  };
}

export interface ProposalAnalysis {
  proposalId: string;
  daoName: string;
  title: string;
  description: string;
  proposalType: 'governance' | 'treasury' | 'technical';
  aiDecision: AIDecision;
  votingDeadline: Date;
  currentVotes: {
    for: number;
    against: number;
    abstain: number;
  };
}

export interface MarketAnalysis {
  tokenA: string;
  tokenB: string;
  currentPrice: number;
  priceChange24h: number;
  volume24h: number;
  liquidity: number;
  aiDecision: AIDecision;
  recommendedAction: 'BUY' | 'SELL' | 'HOLD' | 'WAIT';
  confidenceScore: number;
}

export interface NetworkOptimization {
  currentGasPrice: number;
  recommendedGasPrice: number;
  optimalTiming: Date;
  estimatedSavings: number;
  networkCongestion: 'low' | 'medium' | 'high';
  aiDecision: AIDecision;
}

/**
 * AI Client for frontend integration
 */
export class MetaPilotAIClient {
  private baseUrl: string;
  private apiKey: string;

  constructor(baseUrl?: string, apiKey?: string) {
    this.baseUrl = baseUrl || process.env.NEXT_PUBLIC_AI_ENGINE_URL || 'http://localhost:3001';
    this.apiKey = apiKey || process.env.NEXT_PUBLIC_AI_ENGINE_API_KEY || '';
  }

  private async makeRequest(endpoint: string, data: any): Promise<any> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': this.apiKey ? `Bearer ${this.apiKey}` : '',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        throw new Error(`AI Engine request failed: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      // Fallback to mock data for development
      console.warn('AI Engine not available, using mock data:', error);
      return this.getMockResponse(endpoint, data);
    }
  }

  private getMockResponse(endpoint: string, data: any): any {
    // Mock responses for development and testing
    switch (endpoint) {
      case '/analyze/proposal':
        return {
          success: true,
          decision: {
            action: 'EXECUTE',
            confidence: 85,
            reasoning: [
              'Proposal aligns with governance best practices',
              'Community sentiment is positive (78% approval)',
              'Technical implementation appears sound',
              'No red flags detected in proposal text'
            ],
            metadata: {
              vote: 'FOR',
              riskLevel: 'low',
              communitySupport: 78,
            }
          }
        };

      case '/analyze/market':
        return {
          success: true,
          decision: {
            action: 'WAIT',
            confidence: 72,
            reasoning: [
              'Market volatility is currently high',
              'Technical indicators suggest downward trend',
              'Recommend waiting for price stabilization',
              'Liquidity appears sufficient for execution'
            ],
            metadata: {
              swapTiming: 'LATER',
              recommendedPrice: data.currentPrice * 0.95,
              volatilityScore: 85,
            }
          }
        };

      case '/analyze/network':
        return {
          success: true,
          decision: {
            action: 'EXECUTE',
            confidence: 90,
            reasoning: [
              'Current network congestion is low',
              'Gas prices are near daily minimum',
              'Optimal execution window identified',
              'Potential savings: 15-25%'
            ],
            metadata: {
              gasOptimization: {
                suggestedGasPrice: 4500,
                optimalTiming: new Date(Date.now() + 300000), // 5 minutes
                estimatedSavings: 18,
              }
            }
          }
        };

      default:
        return {
          success: false,
          error: 'Unknown endpoint'
        };
    }
  }

  // ============== DAO PROPOSAL ANALYSIS ==============

  async analyzeProposal(
    proposalData: {
      proposalId: string;
      daoName: string;
      proposalText: string;
      proposalType: string;
      currentVotes?: { for: number; against: number; abstain: number };
      deadline?: Date;
    },
    userRules: AutomationRule[],
    context?: any
  ): Promise<{
    success: boolean;
    analysis?: ProposalAnalysis;
    error?: string;
  }> {
    try {
      const response = await this.makeRequest('/analyze/proposal', {
        ...proposalData,
        userRules: userRules.filter(r => r.type === 'dao_voting' && r.enabled),
        context,
      });

      if (!response.success) {
        return { success: false, error: response.error || 'Analysis failed' };
      }

      const analysis: ProposalAnalysis = {
        proposalId: proposalData.proposalId,
        daoName: proposalData.daoName,
        title: proposalData.proposalText.split('\n')[0] || 'Untitled Proposal',
        description: proposalData.proposalText,
        proposalType: proposalData.proposalType as 'governance' | 'treasury' | 'technical',
        aiDecision: response.decision,
        votingDeadline: proposalData.deadline || new Date(),
        currentVotes: proposalData.currentVotes || { for: 0, against: 0, abstain: 0 },
      };

      return { success: true, analysis };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Analysis failed' 
      };
    }
  }

  // ============== MARKET ANALYSIS ==============

  async analyzeMarket(
    marketData: {
      tokenA: string;
      tokenB: string;
      currentPrice: number;
      priceChange24h: number;
      volume24h: number;
      liquidity: number;
      amount?: number;
    },
    userRules: AutomationRule[],
    context?: any
  ): Promise<{
    success: boolean;
    analysis?: MarketAnalysis;
    error?: string;
  }> {
    try {
      const response = await this.makeRequest('/analyze/market', {
        ...marketData,
        userRules: userRules.filter(r => r.type === 'token_swap' && r.enabled),
        context,
      });

      if (!response.success) {
        return { success: false, error: response.error || 'Market analysis failed' };
      }

      const analysis: MarketAnalysis = {
        tokenA: marketData.tokenA,
        tokenB: marketData.tokenB,
        currentPrice: marketData.currentPrice,
        priceChange24h: marketData.priceChange24h,
        volume24h: marketData.volume24h,
        liquidity: marketData.liquidity,
        aiDecision: response.decision,
        recommendedAction: this.mapActionToRecommendation(response.decision.action),
        confidenceScore: response.decision.confidence,
      };

      return { success: true, analysis };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Market analysis failed' 
      };
    }
  }

  // ============== NETWORK OPTIMIZATION ==============

  async analyzeNetwork(
    networkData: {
      blockchain: 'ethereum' | 'solana';
      transactionType: string;
      priority: 'low' | 'medium' | 'high';
      currentGasPrice?: number;
      estimatedGas?: number;
    },
    userRules: AutomationRule[],
    context?: any
  ): Promise<{
    success: boolean;
    optimization?: NetworkOptimization;
    error?: string;
  }> {
    try {
      const response = await this.makeRequest('/analyze/network', {
        ...networkData,
        userRules: userRules.filter(r => r.type === 'gas_optimization' && r.enabled),
        context,
      });

      if (!response.success) {
        return { success: false, error: response.error || 'Network analysis failed' };
      }

      const gasOptimization = response.decision.metadata.gasOptimization;
      const optimization: NetworkOptimization = {
        currentGasPrice: networkData.currentGasPrice || 5000,
        recommendedGasPrice: gasOptimization?.suggestedGasPrice || 5000,
        optimalTiming: gasOptimization?.optimalTiming || new Date(),
        estimatedSavings: gasOptimization?.estimatedSavings || 0,
        networkCongestion: this.getCongestionLevel(response.decision.confidence),
        aiDecision: response.decision,
      };

      return { success: true, optimization };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Network analysis failed' 
      };
    }
  }

  // ============== YIELD OPTIMIZATION ==============

  async analyzeYieldStrategy(
    strategyData: {
      currentAllocations: number[];
      totalValue: number;
      currentYield: number;
      protocols: string[];
      riskProfile: 'conservative' | 'moderate' | 'aggressive';
    },
    userRules: AutomationRule[],
    context?: any
  ): Promise<{
    success: boolean;
    recommendation?: {
      shouldRebalance: boolean;
      recommendedAllocations?: number[];
      expectedYieldIncrease?: number;
      riskAssessment: string;
      aiDecision: AIDecision;
    };
    error?: string;
  }> {
    try {
      const response = await this.makeRequest('/analyze/yield', {
        ...strategyData,
        userRules: userRules.filter(r => r.type === 'yield_optimization' && r.enabled),
        context,
      });

      if (!response.success) {
        return { success: false, error: response.error || 'Yield analysis failed' };
      }

      const recommendation = {
        shouldRebalance: response.decision.action === 'EXECUTE',
        recommendedAllocations: response.decision.metadata.allocations,
        expectedYieldIncrease: response.decision.metadata.yieldIncrease || 0,
        riskAssessment: response.decision.metadata.riskLevel || 'medium',
        aiDecision: response.decision,
      };

      return { success: true, recommendation };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Yield analysis failed' 
      };
    }
  }

  // ============== AUTOMATION MANAGEMENT ==============

  async createAutomationRule(rule: Omit<AutomationRule, 'id'>): Promise<{
    success: boolean;
    ruleId?: string;
    error?: string;
  }> {
    try {
      const ruleWithId = {
        ...rule,
        id: `rule-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      };

      // Store in localStorage for now (in production, this would be backend)
      const existingRules = this.getStoredRules();
      existingRules.push(ruleWithId);
      localStorage.setItem('metapilot-automation-rules', JSON.stringify(existingRules));

      return { success: true, ruleId: ruleWithId.id };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to create rule' 
      };
    }
  }

  async getAutomationRules(): Promise<AutomationRule[]> {
    return this.getStoredRules();
  }

  async updateAutomationRule(ruleId: string, updates: Partial<AutomationRule>): Promise<{
    success: boolean;
    error?: string;
  }> {
    try {
      const rules = this.getStoredRules();
      const ruleIndex = rules.findIndex(r => r.id === ruleId);
      
      if (ruleIndex === -1) {
        return { success: false, error: 'Rule not found' };
      }

      rules[ruleIndex] = { ...rules[ruleIndex], ...updates };
      localStorage.setItem('metapilot-automation-rules', JSON.stringify(rules));

      return { success: true };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to update rule' 
      };
    }
  }

  async deleteAutomationRule(ruleId: string): Promise<{
    success: boolean;
    error?: string;
  }> {
    try {
      const rules = this.getStoredRules();
      const filteredRules = rules.filter(r => r.id !== ruleId);
      localStorage.setItem('metapilot-automation-rules', JSON.stringify(filteredRules));

      return { success: true };
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to delete rule' 
      };
    }
  }

  // ============== UTILITY METHODS ==============

  private getStoredRules(): AutomationRule[] {
    if (typeof localStorage === 'undefined') return [];
    
    try {
      const stored = localStorage.getItem('metapilot-automation-rules');
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  private mapActionToRecommendation(action: string): 'BUY' | 'SELL' | 'HOLD' | 'WAIT' {
    switch (action) {
      case 'EXECUTE': return 'BUY';
      case 'WAIT': return 'WAIT';
      case 'REJECT': return 'HOLD';
      default: return 'WAIT';
    }
  }

  private getCongestionLevel(confidence: number): 'low' | 'medium' | 'high' {
    if (confidence > 80) return 'low';
    if (confidence > 60) return 'medium';
    return 'high';
  }
}

// Export singleton instance
export const aiClient = new MetaPilotAIClient();

// Export hook for React components
export function useAIClient() {
  return aiClient;
}