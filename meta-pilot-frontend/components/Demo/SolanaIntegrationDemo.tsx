/**
 * MetaPilot Solana Integration Demo
 * Showcases all smart contract functionalities
 */

'use client';

import React, { useState, useEffect } from 'react';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, CheckCircle, XCircle, Clock } from 'lucide-react';

// Import our custom hooks and utilities
import { useMetaPilotClient, useDAOProposals, useConnectionHealth } from '@/hooks/use-solana-programs';
import { SOLANA_CONFIG, validateConfig } from '@/lib/config';

interface ConnectionStatus {
  isConnected: boolean;
  network: string;
  health: 'healthy' | 'degraded' | 'offline';
  latency: number;
}

export default function SolanaIntegrationDemo() {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
    isConnected: false,
    network: SOLANA_CONFIG.NETWORK,
    health: 'offline',
    latency: -1,
  });

  const [configValid, setConfigValid] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);

  // Initialize MetaPilot client
  const client = useMetaPilotClient(SOLANA_CONFIG.RPC_URL, SOLANA_CONFIG.NETWORK);
  const { health } = useConnectionHealth(client);
  const { proposals, loading: proposalsLoading } = useDAOProposals(client);

  // Validate configuration on mount
  useEffect(() => {
    try {
      validateConfig();
      setConfigValid(true);
      setConfigError(null);
    } catch (error) {
      setConfigValid(false);
      setConfigError(error instanceof Error ? error.message : 'Configuration validation failed');
    }
  }, []);

  // Update connection status
  useEffect(() => {
    if (health) {
      setConnectionStatus({
        isConnected: health.health !== 'offline',
        network: health.cluster,
        health: health.health,
        latency: health.latency,
      });
    }
  }, [health]);

  const getHealthColor = (health: string) => {
    switch (health) {
      case 'healthy': return 'text-green-600';
      case 'degraded': return 'text-yellow-600';
      case 'offline': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  const getHealthIcon = (health: string) => {
    switch (health) {
      case 'healthy': return <CheckCircle className="w-4 h-4" />;
      case 'degraded': return <Clock className="w-4 h-4" />;
      case 'offline': return <XCircle className="w-4 h-4" />;
      default: return <Loader2 className="w-4 h-4 animate-spin" />;
    }
  };

  return (
    <div className="container mx-auto py-8 space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold">MetaPilot Solana Integration</h1>
        <p className="text-muted-foreground">
          Smart contract integration dashboard - Ready for deployment
        </p>
      </div>

      {/* Configuration Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Configuration Status
            {configValid ? (
              <Badge variant="default" className="bg-green-100 text-green-800">
                Valid
              </Badge>
            ) : (
              <Badge variant="destructive">Invalid</Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {!configValid && configError && (
            <Alert className="mb-4">
              <AlertDescription>{configError}</AlertDescription>
            </Alert>
          )}
          
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="space-y-1">
              <p className="text-sm font-medium">Network</p>
              <p className="text-sm text-muted-foreground">{SOLANA_CONFIG.NETWORK}</p>
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">RPC URL</p>
              <p className="text-sm text-muted-foreground truncate">{SOLANA_CONFIG.RPC_URL}</p>
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">Programs</p>
              <p className="text-sm text-muted-foreground">
                {Object.keys(SOLANA_CONFIG.PROGRAM_IDS).length} configured
              </p>
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">AI Engine</p>
              <p className="text-sm text-muted-foreground">
                {process.env.NEXT_PUBLIC_AI_ENGINE_ENABLED === 'true' ? 'Enabled' : 'Disabled'}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Connection Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Connection Status
            <div className={`flex items-center gap-1 ${getHealthColor(connectionStatus.health)}`}>
              {getHealthIcon(connectionStatus.health)}
              <span className="capitalize">{connectionStatus.health}</span>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            <div className="space-y-1">
              <p className="text-sm font-medium">Status</p>
              <p className="text-sm text-muted-foreground">
                {connectionStatus.isConnected ? 'Connected' : 'Disconnected'}
              </p>
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">Network</p>
              <p className="text-sm text-muted-foreground">{connectionStatus.network}</p>
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">Latency</p>
              <p className="text-sm text-muted-foreground">
                {connectionStatus.latency > 0 ? `${connectionStatus.latency}ms` : 'N/A'}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Program Information */}
      <Card>
        <CardHeader>
          <CardTitle>Smart Contract Programs</CardTitle>
          <CardDescription>
            Deployed program addresses and their capabilities
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="dao-voting" className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="dao-voting">DAO Voting</TabsTrigger>
              <TabsTrigger value="token-automation">Token Automation</TabsTrigger>
              <TabsTrigger value="yield-optimizer">Yield Optimizer</TabsTrigger>
              <TabsTrigger value="transaction-scheduler">Transaction Scheduler</TabsTrigger>
            </TabsList>

            <TabsContent value="dao-voting" className="space-y-4">
              <div className="space-y-2">
                <h3 className="text-lg font-semibold">DAO Voting Program</h3>
                <p className="text-sm text-muted-foreground">
                  Automated proposal analysis and voting based on user-defined rules
                </p>
                <div className="bg-muted p-3 rounded-md">
                  <p className="text-sm font-mono">{SOLANA_CONFIG.PROGRAM_IDS.DAO_VOTING}</p>
                </div>
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div>
                    <h4 className="font-medium mb-1">Capabilities</h4>
                    <ul className="text-sm text-muted-foreground space-y-1">
                      <li>• Create proposals</li>
                      <li>• Automated voting</li>
                      <li>• AI-powered analysis</li>
                      <li>• Gas optimization</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">Status</h4>
                    <div className="space-y-1">
                      <Badge variant="outline">Ready for deployment</Badge>
                      <p className="text-sm text-muted-foreground">
                        Proposals loaded: {proposalsLoading ? 'Loading...' : proposals.length}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="token-automation" className="space-y-4">
              <div className="space-y-2">
                <h3 className="text-lg font-semibold">Token Automation Program</h3>
                <p className="text-sm text-muted-foreground">
                  Automated token swaps with AI-optimized timing and price analysis
                </p>
                <div className="bg-muted p-3 rounded-md">
                  <p className="text-sm font-mono">{SOLANA_CONFIG.PROGRAM_IDS.TOKEN_AUTOMATION}</p>
                </div>
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div>
                    <h4 className="font-medium mb-1">Capabilities</h4>
                    <ul className="text-sm text-muted-foreground space-y-1">
                      <li>• Automated swaps</li>
                      <li>• Price threshold monitoring</li>
                      <li>• Slippage protection</li>
                      <li>• Multi-DEX routing</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">Status</h4>
                    <Badge variant="outline">Ready for deployment</Badge>
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="yield-optimizer" className="space-y-4">
              <div className="space-y-2">
                <h3 className="text-lg font-semibold">Yield Optimizer Program</h3>
                <p className="text-sm text-muted-foreground">
                  AI-driven yield farming optimization across multiple Solana DeFi protocols
                </p>
                <div className="bg-muted p-3 rounded-md">
                  <p className="text-sm font-mono">{SOLANA_CONFIG.PROGRAM_IDS.YIELD_OPTIMIZER}</p>
                </div>
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div>
                    <h4 className="font-medium mb-1">Capabilities</h4>
                    <ul className="text-sm text-muted-foreground space-y-1">
                      <li>• Strategy creation</li>
                      <li>• Auto-rebalancing</li>
                      <li>• Yield harvesting</li>
                      <li>• Risk management</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">Status</h4>
                    <Badge variant="outline">Ready for deployment</Badge>
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="transaction-scheduler" className="space-y-4">
              <div className="space-y-2">
                <h3 className="text-lg font-semibold">Transaction Scheduler Program</h3>
                <p className="text-sm text-muted-foreground">
                  Schedule and execute transactions at optimal times with gas optimization
                </p>
                <div className="bg-muted p-3 rounded-md">
                  <p className="text-sm font-mono">{SOLANA_CONFIG.PROGRAM_IDS.TRANSACTION_SCHEDULER}</p>
                </div>
                <div className="grid grid-cols-2 gap-4 pt-2">
                  <div>
                    <h4 className="font-medium mb-1">Capabilities</h4>
                    <ul className="text-sm text-muted-foreground space-y-1">
                      <li>• Schedule transactions</li>
                      <li>• Time-based execution</li>
                      <li>• Gas price optimization</li>
                      <li>• Conditional execution</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">Status</h4>
                    <Badge variant="outline">Ready for deployment</Badge>
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* Integration Status */}
      <Card>
        <CardHeader>
          <CardTitle>Frontend Integration Status</CardTitle>
          <CardDescription>
            Ready for smart contract deployment and frontend connection
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <h4 className="font-medium">✅ Completed Integration</h4>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• Program interface definitions</li>
                  <li>• React hooks for smart contracts</li>
                  <li>• AI engine integration layer</li>
                  <li>• Multi-chain wallet support</li>
                  <li>• Configuration management</li>
                  <li>• Deployment scripts</li>
                </ul>
              </div>
              <div className="space-y-2">
                <h4 className="font-medium">🚀 Next Steps</h4>
                <ul className="text-sm text-muted-foreground space-y-1">
                  <li>• Deploy smart contracts to Solana</li>
                  <li>• Update environment configuration</li>
                  <li>• Test program interactions</li>
                  <li>• Enable AI automation features</li>
                  <li>• Connect wallet and start trading</li>
                </ul>
              </div>
            </div>
            
            <Alert>
              <AlertDescription>
                <strong>Ready for deployment!</strong> Run the deployment script to deploy your smart contracts:
                <br />
                <code className="bg-muted px-2 py-1 rounded mt-2 inline-block">
                  cd metapilot-programs/metapilot-core && ./deploy.sh devnet
                </code>
              </AlertDescription>
            </Alert>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
