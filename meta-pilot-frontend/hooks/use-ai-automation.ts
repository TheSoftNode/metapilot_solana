"use client";

import { useState, useEffect, useCallback } from 'react';
import { aiClient, AIDecision, AutomationRule, ProposalAnalysis, MarketAnalysis, NetworkOptimization } from '@/lib/ai-client';
import { useMultiChainWallet } from './use-multi-chain-wallet';

export interface AIAutomationState {
  isAnalyzing: boolean;
  lastAnalysis: Date | null;
  error: string | null;
  automationRules: AutomationRule[];
  activeDecisions: AIDecision[];
}

export interface DAOProposal {
  id: string;
  daoName: string;
  title: string;
  description: string;
  type: 'governance' | 'treasury' | 'technical';
  votingDeadline: Date;
  currentVotes: {
    for: number;
    against: number;
    abstain: number;
  };
  userHasVoted: boolean;
  blockchain: 'ethereum' | 'solana';
}

export function useAIAutomation() {
  const { isConnected, ethereum, solana } = useMultiChainWallet();
  const [state, setState] = useState<AIAutomationState>({
    isAnalyzing: false,
    lastAnalysis: null,
    error: null,
    automationRules: [],
    activeDecisions: [],
  });

  // Load automation rules on mount
  useEffect(() => {
    const loadRules = async () => {
      try {
        const rules = await aiClient.getAutomationRules();
        setState(prev => ({ ...prev, automationRules: rules }));
      } catch (error) {
        console.error('Failed to load automation rules:', error);
      }
    };

    if (isConnected) {
      loadRules();
    }
  }, [isConnected]);

  // ============== PROPOSAL ANALYSIS ==============

  const analyzeProposal = useCallback(async (proposal: DAOProposal): Promise<{
    success: boolean;
    analysis?: ProposalAnalysis;
    shouldAutoVote?: boolean;
    error?: string;
  }> => {
    if (!isConnected) {
      return { success: false, error: 'Wallet not connected' };
    }

    setState(prev => ({ ...prev, isAnalyzing: true, error: null }));

    try {
      const result = await aiClient.analyzeProposal(
        {
          proposalId: proposal.id,
          daoName: proposal.daoName,
          proposalText: `${proposal.title}\n\n${proposal.description}`,
          proposalType: proposal.type,
          currentVotes: proposal.currentVotes,
          deadline: proposal.votingDeadline,
        },
        state.automationRules,
        {
          blockchain: proposal.blockchain,
          userWallet: proposal.blockchain === 'ethereum' ? ethereum.address : solana.address,
          hasVoted: proposal.userHasVoted,
        }
      );

      if (result.success && result.analysis) {
        const shouldAutoVote = result.analysis.aiDecision.action === 'EXECUTE' && 
                             result.analysis.aiDecision.confidence >= 70 &&
                             !proposal.userHasVoted;

        setState(prev => ({
          ...prev,
          isAnalyzing: false,
          lastAnalysis: new Date(),
          activeDecisions: [...prev.activeDecisions, result.analysis!.aiDecision],
        }));

        return { success: true, analysis: result.analysis, shouldAutoVote };
      }

      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: result.error || 'Analysis failed' 
      }));

      return { success: false, error: result.error };
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: error instanceof Error ? error.message : 'Analysis failed' 
      }));

      return { success: false, error: error instanceof Error ? error.message : 'Analysis failed' };
    }
  }, [isConnected, ethereum.address, solana.address, state.automationRules]);

  // ============== MARKET ANALYSIS ==============

  const analyzeSwapTiming = useCallback(async (
    tokenA: string,
    tokenB: string,
    amount: number,
    blockchain: 'ethereum' | 'solana' = 'ethereum'
  ): Promise<{
    success: boolean;
    analysis?: MarketAnalysis;
    shouldExecute?: boolean;
    error?: string;
  }> => {
    if (!isConnected) {
      return { success: false, error: 'Wallet not connected' };
    }

    setState(prev => ({ ...prev, isAnalyzing: true, error: null }));

    try {
      // In a real implementation, you'd fetch market data from APIs
      const mockMarketData = {
        tokenA,
        tokenB,
        currentPrice: 1.0,
        priceChange24h: 2.5,
        volume24h: 1000000,
        liquidity: 5000000,
        amount,
      };

      const result = await aiClient.analyzeMarket(
        mockMarketData,
        state.automationRules,
        {
          blockchain,
          userWallet: blockchain === 'ethereum' ? ethereum.address : solana.address,
          amount,
        }
      );

      if (result.success && result.analysis) {
        const shouldExecute = result.analysis.aiDecision.action === 'EXECUTE' && 
                            result.analysis.aiDecision.confidence >= 75;

        setState(prev => ({
          ...prev,
          isAnalyzing: false,
          lastAnalysis: new Date(),
          activeDecisions: [...prev.activeDecisions, result.analysis!.aiDecision],
        }));

        return { success: true, analysis: result.analysis, shouldExecute };
      }

      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: result.error || 'Market analysis failed' 
      }));

      return { success: false, error: result.error };
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: error instanceof Error ? error.message : 'Market analysis failed' 
      }));

      return { success: false, error: error instanceof Error ? error.message : 'Analysis failed' };
    }
  }, [isConnected, ethereum.address, solana.address, state.automationRules]);

  // ============== NETWORK OPTIMIZATION ==============

  const optimizeTransaction = useCallback(async (
    transactionType: string,
    blockchain: 'ethereum' | 'solana' = 'ethereum',
    priority: 'low' | 'medium' | 'high' = 'medium'
  ): Promise<{
    success: boolean;
    optimization?: NetworkOptimization;
    shouldWait?: boolean;
    error?: string;
  }> => {
    if (!isConnected) {
      return { success: false, error: 'Wallet not connected' };
    }

    setState(prev => ({ ...prev, isAnalyzing: true, error: null }));

    try {
      const result = await aiClient.analyzeNetwork(
        {
          blockchain,
          transactionType,
          priority,
          currentGasPrice: blockchain === 'ethereum' ? 20000000000 : 5000, // 20 gwei for ETH, 5000 lamports for SOL
        },
        state.automationRules,
        {
          userWallet: blockchain === 'ethereum' ? ethereum.address : solana.address,
          priority,
        }
      );

      if (result.success && result.optimization) {
        const shouldWait = result.optimization.aiDecision.action === 'WAIT' ||
                         result.optimization.aiDecision.metadata.swapTiming === 'LATER';

        setState(prev => ({
          ...prev,
          isAnalyzing: false,
          lastAnalysis: new Date(),
          activeDecisions: [...prev.activeDecisions, result.optimization!.aiDecision],
        }));

        return { success: true, optimization: result.optimization, shouldWait };
      }

      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: result.error || 'Network optimization failed' 
      }));

      return { success: false, error: result.error };
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        isAnalyzing: false, 
        error: error instanceof Error ? error.message : 'Network optimization failed' 
      }));

      return { success: false, error: error instanceof Error ? error.message : 'Analysis failed' };
    }
  }, [isConnected, ethereum.address, solana.address, state.automationRules]);

  // ============== AUTOMATION RULES MANAGEMENT ==============

  const createRule = useCallback(async (rule: Omit<AutomationRule, 'id'>): Promise<{
    success: boolean;
    ruleId?: string;
    error?: string;
  }> => {
    try {
      const result = await aiClient.createAutomationRule(rule);
      
      if (result.success) {
        // Refresh rules
        const updatedRules = await aiClient.getAutomationRules();
        setState(prev => ({ ...prev, automationRules: updatedRules }));
      }

      return result;
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to create rule' 
      };
    }
  }, []);

  const updateRule = useCallback(async (ruleId: string, updates: Partial<AutomationRule>): Promise<{
    success: boolean;
    error?: string;
  }> => {
    try {
      const result = await aiClient.updateAutomationRule(ruleId, updates);
      
      if (result.success) {
        // Refresh rules
        const updatedRules = await aiClient.getAutomationRules();
        setState(prev => ({ ...prev, automationRules: updatedRules }));
      }

      return result;
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to update rule' 
      };
    }
  }, []);

  const deleteRule = useCallback(async (ruleId: string): Promise<{
    success: boolean;
    error?: string;
  }> => {
    try {
      const result = await aiClient.deleteAutomationRule(ruleId);
      
      if (result.success) {
        // Refresh rules
        const updatedRules = await aiClient.getAutomationRules();
        setState(prev => ({ ...prev, automationRules: updatedRules }));
      }

      return result;
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Failed to delete rule' 
      };
    }
  }, []);

  const toggleRule = useCallback(async (ruleId: string): Promise<void> => {
    const rule = state.automationRules.find(r => r.id === ruleId);
    if (rule) {
      await updateRule(ruleId, { enabled: !rule.enabled });
    }
  }, [state.automationRules, updateRule]);

  // ============== BULK ANALYSIS ==============

  const analyzeMultipleProposals = useCallback(async (proposals: DAOProposal[]): Promise<{
    analyses: (ProposalAnalysis | null)[];
    autoVoteRecommendations: string[]; // proposal IDs recommended for auto-voting
    errors: string[];
  }> => {
    const analyses: (ProposalAnalysis | null)[] = [];
    const autoVoteRecommendations: string[] = [];
    const errors: string[] = [];

    for (const proposal of proposals) {
      try {
        const result = await analyzeProposal(proposal);
        
        if (result.success && result.analysis) {
          analyses.push(result.analysis);
          
          if (result.shouldAutoVote) {
            autoVoteRecommendations.push(proposal.id);
          }
        } else {
          analyses.push(null);
          if (result.error) {
            errors.push(`${proposal.id}: ${result.error}`);
          }
        }
      } catch (error) {
        analyses.push(null);
        errors.push(`${proposal.id}: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }

      // Small delay to avoid overwhelming the AI service
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    return { analyses, autoVoteRecommendations, errors };
  }, [analyzeProposal]);

  // ============== UTILITY FUNCTIONS ==============

  const clearError = useCallback(() => {
    setState(prev => ({ ...prev, error: null }));
  }, []);

  const clearActiveDecisions = useCallback(() => {
    setState(prev => ({ ...prev, activeDecisions: [] }));
  }, []);

  const getEnabledRulesByType = useCallback((type: AutomationRule['type']) => {
    return state.automationRules.filter(rule => rule.type === type && rule.enabled);
  }, [state.automationRules]);

  return {
    // State
    ...state,
    isConnected,
    
    // Analysis functions
    analyzeProposal,
    analyzeSwapTiming,
    optimizeTransaction,
    analyzeMultipleProposals,
    
    // Rule management
    createRule,
    updateRule,
    deleteRule,
    toggleRule,
    getEnabledRulesByType,
    
    // Utility
    clearError,
    clearActiveDecisions,
  };
}