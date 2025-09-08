/**
 * MetaPilot Solana Hooks
 * React hooks for easy integration with MetaPilot smart contracts
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { 
  MetaPilotClient, 
  createMetaPilotClient,
  CreateProposalParams,
  CreateSwapConfigParams,
  CreateYieldStrategyParams,
  ScheduleTransactionParams 
} from '../lib/metapilot-client';
import { 
  ProposalData, 
  SwapConfigData, 
  YieldStrategyData, 
  ScheduledTransactionData 
} from '../lib/solana-programs';

// Hook for MetaPilot client
export function useMetaPilotClient(
  rpcUrl?: string,
  cluster: 'localnet' | 'devnet' | 'mainnet-beta' = 'devnet'
) {
  const client = useMemo(() => {
    const url = rpcUrl || 
      (cluster === 'localnet' ? 'http://127.0.0.1:8899' :
       cluster === 'devnet' ? 'https://api.devnet.solana.com' :
       'https://api.mainnet-beta.solana.com');
    
    return createMetaPilotClient(url, cluster);
  }, [rpcUrl, cluster]);

  return client;
}

// Hook for connection health monitoring
export function useConnectionHealth(client: MetaPilotClient) {
  const [health, setHealth] = useState<{
    cluster: string;
    health: 'healthy' | 'degraded' | 'offline';
    latency: number;
  } | null>(null);

  const checkHealth = useCallback(async () => {
    try {
      const healthStatus = await client.getConnectionHealth();
      setHealth(healthStatus);
    } catch (error) {
      setHealth({
        cluster: 'unknown',
        health: 'offline',
        latency: -1,
      });
    }
  }, [client]);

  useEffect(() => {
    checkHealth();
    const interval = setInterval(checkHealth, 30000); // Check every 30 seconds
    
    return () => clearInterval(interval);
  }, [checkHealth]);

  return { health, refetch: checkHealth };
}

// Hook for DAO proposals
export function useDAOProposals(client: MetaPilotClient) {
  const [proposals, setProposals] = useState<ProposalData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProposals = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const proposals = await client.getAllProposals();
      setProposals(proposals);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch proposals');
    } finally {
      setLoading(false);
    }
  }, [client]);

  const createProposal = useCallback(async (
    params: CreateProposalParams,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const result = await client.createProposal(params, wallet);
      await fetchProposals(); // Refresh list
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create proposal');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client, fetchProposals]);

  const castVote = useCallback(async (
    proposalId: number,
    support: boolean,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const signature = await client.castVote(proposalId, support, wallet);
      await fetchProposals(); // Refresh list
      return signature;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cast vote');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client, fetchProposals]);

  useEffect(() => {
    fetchProposals();
  }, [fetchProposals]);

  return {
    proposals,
    loading,
    error,
    createProposal,
    castVote,
    refetch: fetchProposals,
  };
}

// Hook for token automation
export function useTokenAutomation(client: MetaPilotClient, userPublicKey?: PublicKey) {
  const [swapConfigs, setSwapConfigs] = useState<SwapConfigData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createSwapConfig = useCallback(async (
    params: CreateSwapConfigParams,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const result = await client.createSwapConfig(params, wallet);
      // Refresh user configs
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create swap config');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  const executeSwap = useCallback(async (
    inputMint: PublicKey,
    inputTokenAccount: PublicKey,
    outputTokenAccount: PublicKey,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const signature = await client.executeSwap(inputMint, inputTokenAccount, outputTokenAccount, wallet);
      return signature;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to execute swap');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  return {
    swapConfigs,
    loading,
    error,
    createSwapConfig,
    executeSwap,
  };
}

// Hook for yield strategies
export function useYieldStrategies(client: MetaPilotClient, userPublicKey?: PublicKey) {
  const [strategies, setStrategies] = useState<YieldStrategyData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createStrategy = useCallback(async (
    params: CreateYieldStrategyParams,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const result = await client.createYieldStrategy(params, wallet);
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create yield strategy');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  const deposit = useCallback(async (
    strategyType: number,
    tokenAccount: PublicKey,
    amount: number,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const signature = await client.depositToStrategy(strategyType, tokenAccount, amount, wallet);
      return signature;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to deposit to strategy');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  return {
    strategies,
    loading,
    error,
    createStrategy,
    deposit,
  };
}

// Hook for transaction scheduling
export function useTransactionScheduler(client: MetaPilotClient, userPublicKey?: PublicKey) {
  const [scheduledTransactions, setScheduledTransactions] = useState<ScheduledTransactionData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const scheduleTransaction = useCallback(async (
    params: ScheduleTransactionParams,
    wallet: Keypair
  ) => {
    try {
      setLoading(true);
      const result = await client.scheduleTransaction(params, wallet);
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to schedule transaction');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [client]);

  return {
    scheduledTransactions,
    loading,
    error,
    scheduleTransaction,
  };
}

// Hook for user dashboard data
export function useUserDashboard(client: MetaPilotClient, userPublicKey?: PublicKey) {
  const [dashboardData, setDashboardData] = useState<{
    proposals: ProposalData[];
    swapConfigs: SwapConfigData[];
    yieldStrategies: YieldStrategyData[];
    scheduledTransactions: ScheduledTransactionData[];
  } | null>(null);
  
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDashboardData = useCallback(async () => {
    if (!userPublicKey) return;

    setLoading(true);
    setError(null);
    try {
      const data = await client.getUserAccounts(userPublicKey);
      setDashboardData(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch dashboard data');
    } finally {
      setLoading(false);
    }
  }, [client, userPublicKey]);

  useEffect(() => {
    fetchDashboardData();
  }, [fetchDashboardData]);

  return {
    dashboardData,
    loading,
    error,
    refetch: fetchDashboardData,
  };
}

// Hook for transaction fee estimation
export function useTransactionFee(client: MetaPilotClient) {
  const [estimating, setEstimating] = useState(false);

  const estimateFee = useCallback(async (transaction: any) => {
    setEstimating(true);
    try {
      const fee = await client.estimateTransactionFee(transaction);
      return fee;
    } catch (error) {
      console.error('Error estimating fee:', error);
      return 5000; // Default fallback
    } finally {
      setEstimating(false);
    }
  }, [client]);

  return { estimateFee, estimating };
}

// Hook for real-time account monitoring
export function useAccountMonitor(
  client: MetaPilotClient,
  accountPublicKey: PublicKey | null,
  accountType: 'proposal' | 'swap_config' | 'yield_strategy' | 'scheduled_tx'
) {
  const [accountData, setAccountData] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAccountData = useCallback(async () => {
    if (!accountPublicKey) return;

    setLoading(true);
    setError(null);
    try {
      // This would need to be implemented based on account type
      // For now, just log the attempt
      console.log(`Monitoring ${accountType} account:`, accountPublicKey.toString());
      setAccountData({ lastUpdated: Date.now() });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch account data');
    } finally {
      setLoading(false);
    }
  }, [client, accountPublicKey, accountType]);

  useEffect(() => {
    if (accountPublicKey) {
      fetchAccountData();
      
      // Set up polling for real-time updates
      const interval = setInterval(fetchAccountData, 5000); // Poll every 5 seconds
      
      return () => clearInterval(interval);
    }
  }, [fetchAccountData, accountPublicKey]);

  return {
    accountData,
    loading,
    error,
    refetch: fetchAccountData,
  };
}
