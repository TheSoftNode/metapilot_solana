"use client";

import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Brain, 
  TrendingUp, 
  TrendingDown, 
  AlertTriangle, 
  CheckCircle, 
  Clock, 
  Zap,
  Target,
  BarChart3,
  Activity
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
// Temporarily comment out to avoid import issues
// import { useAIAutomation } from '@/hooks/use-ai-automation';
// import { useMultiChainWallet } from '@/hooks/use-multi-chain-wallet';
import { cn } from '@/lib/utils';

interface AIInsightCard {
  id: string;
  title: string;
  type: 'opportunity' | 'warning' | 'success' | 'info';
  description: string;
  action?: {
    label: string;
    onClick: () => void;
  };
  confidence: number;
  priority: 'high' | 'medium' | 'low';
  blockchain?: 'ethereum' | 'solana';
}

export const AIInsights: React.FC<{ className?: string }> = ({ className }) => {
  // Mock data for now to avoid hook import issues
  const isConnected = true;
  const ethereum = { isConnected: true };
  const solana = { isConnected: false };
  const isAnalyzing = false;
  const lastAnalysis = new Date();
  const automationRules = [{ id: '1', enabled: true }, { id: '2', enabled: false }];
  const activeDecisions: any[] = [];
  const [insights, setInsights] = useState<AIInsightCard[]>([]);
  const [currentInsightIndex, setCurrentInsightIndex] = useState(0);

  // Generate AI insights based on current state
  useEffect(() => {
    if (!isConnected) {
      setInsights([]);
      return;
    }

    const generateInsights = (): AIInsightCard[] => {
      const newInsights: AIInsightCard[] = [];

      // Multi-chain wallet insight
      if (ethereum.isConnected && solana.isConnected) {
        newInsights.push({
          id: 'multichain-active',
          title: 'Multi-Chain Portfolio Active',
          type: 'success',
          description: `Connected to both Ethereum and Solana. AI can now optimize across both networks for better yields and lower fees.`,
          confidence: 95,
          priority: 'high',
        });
      } else if (!solana.isConnected) {
        newInsights.push({
          id: 'solana-opportunity',
          title: 'Solana Integration Available',
          type: 'opportunity',
          description: 'Enable Solana for access to low-fee transactions and growing DeFi ecosystem.',
          confidence: 90,
          priority: 'medium',
          blockchain: 'solana',
        });
      }

      // Automation rules insights
      const enabledRules = automationRules.filter(rule => rule.enabled);
      if (enabledRules.length === 0) {
        newInsights.push({
          id: 'no-automation',
          title: 'No Active Automation',
          type: 'warning',
          description: 'Set up automation rules to let AI handle your Web3 tasks automatically.',
          action: {
            label: 'Create Rule',
            onClick: () => console.log('Navigate to automation settings'),
          },
          confidence: 85,
          priority: 'high',
        });
      } else {
        newInsights.push({
          id: 'automation-active',
          title: `${enabledRules.length} Rules Active`,
          type: 'success',
          description: `AI is monitoring ${enabledRules.length} automation rule${enabledRules.length > 1 ? 's' : ''} for you.`,
          confidence: 90,
          priority: 'medium',
        });
      }

      // Recent AI decisions insight
      if (activeDecisions.length > 0) {
        const recentDecision = activeDecisions[activeDecisions.length - 1];
        newInsights.push({
          id: 'recent-decision',
          title: 'Recent AI Analysis',
          type: recentDecision.action === 'EXECUTE' ? 'success' : 'info',
          description: `Latest analysis: ${recentDecision.action} (${recentDecision.confidence}% confidence) - ${recentDecision.reasoning[0] || 'Analysis complete'}`,
          confidence: recentDecision.confidence,
          priority: recentDecision.action === 'EXECUTE' ? 'high' : 'medium',
        });
      }

      // Market opportunity insights (mock data)
      const currentHour = new Date().getHours();
      if (currentHour >= 22 || currentHour <= 6) {
        newInsights.push({
          id: 'gas-optimization',
          title: 'Low Gas Period Detected',
          type: 'opportunity',
          description: 'Current gas prices are 25% below average. Consider executing pending transactions.',
          confidence: 78,
          priority: 'medium',
          blockchain: 'ethereum',
        });
      }

      // Solana Pay opportunity
      if (solana.isConnected) {
        newInsights.push({
          id: 'solana-pay-available',
          title: 'Solana Pay Ready',
          type: 'info',
          description: 'Generate QR codes for instant, low-cost payments on Solana network.',
          confidence: 100,
          priority: 'low',
          blockchain: 'solana',
        });
      }

      return newInsights.slice(0, 5); // Limit to 5 insights
    };

    setInsights(generateInsights());
  }, [isConnected, ethereum.isConnected, solana.isConnected, automationRules, activeDecisions]);

  // Auto-rotate insights
  useEffect(() => {
    if (insights.length <= 1) return;

    const interval = setInterval(() => {
      setCurrentInsightIndex(prev => (prev + 1) % insights.length);
    }, 5000);

    return () => clearInterval(interval);
  }, [insights.length]);

  const getInsightIcon = (type: AIInsightCard['type']) => {
    switch (type) {
      case 'opportunity':
        return <TrendingUp className="h-4 w-4 text-green-500" />;
      case 'warning':
        return <AlertTriangle className="h-4 w-4 text-yellow-500" />;
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'info':
      default:
        return <Brain className="h-4 w-4 text-blue-500" />;
    }
  };

  const getInsightBorderColor = (type: AIInsightCard['type']) => {
    switch (type) {
      case 'opportunity':
        return 'border-l-green-500';
      case 'warning':
        return 'border-l-yellow-500';
      case 'success':
        return 'border-l-green-500';
      case 'info':
      default:
        return 'border-l-blue-500';
    }
  };

  if (!isConnected) {
    return (
      <Card className={cn("relative overflow-hidden", className)}>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Brain className="h-5 w-5 text-purple-600 dark:text-purple-400" />
            AI Insights
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Brain className="h-12 w-12 text-gray-400 mb-4" />
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
              Connect your wallet to get AI-powered insights
            </p>
            <p className="text-xs text-gray-500">
              AI analysis will help optimize your Web3 experience
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const currentInsight = insights[currentInsightIndex];
  const stats = {
    totalRules: automationRules.length,
    activeRules: automationRules.filter(r => r.enabled).length,
    totalDecisions: activeDecisions.length,
    lastAnalysisTime: lastAnalysis,
  };

  return (
    <Card className={cn("relative overflow-hidden", className)}>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center justify-between text-lg">
          <div className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-purple-600 dark:text-purple-400" />
            AI Insights
          </div>
          {isAnalyzing && (
            <div className="flex items-center gap-1 text-sm text-blue-600 dark:text-blue-400">
              <motion.div
                animate={{ rotate: 360 }}
                transition={{ duration: 2, repeat: Infinity, ease: "linear" }}
              >
                <Activity className="h-4 w-4" />
              </motion.div>
              Analyzing...
            </div>
          )}
        </CardTitle>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Quick Stats */}
        <div className="grid grid-cols-3 gap-4 pb-4 border-b border-gray-200 dark:border-gray-700">
          <div className="text-center">
            <div className="text-xl font-bold text-purple-600 dark:text-purple-400">
              {stats.activeRules}
            </div>
            <div className="text-xs text-gray-600 dark:text-gray-400">
              Active Rules
            </div>
          </div>
          <div className="text-center">
            <div className="text-xl font-bold text-blue-600 dark:text-blue-400">
              {stats.totalDecisions}
            </div>
            <div className="text-xs text-gray-600 dark:text-gray-400">
              Decisions Made
            </div>
          </div>
          <div className="text-center">
            <div className="text-xl font-bold text-green-600 dark:text-green-400">
              {ethereum.isConnected && solana.isConnected ? 2 : 1}
            </div>
            <div className="text-xs text-gray-600 dark:text-gray-400">
              Chains Active
            </div>
          </div>
        </div>

        {/* Current Insight */}
        <AnimatePresence mode="wait">
          {currentInsight && (
            <motion.div
              key={currentInsight.id}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.3 }}
              className={cn(
                "p-4 rounded-lg border-l-4",
                getInsightBorderColor(currentInsight.type),
                "bg-gray-50 dark:bg-gray-800/50"
              )}
            >
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  {getInsightIcon(currentInsight.type)}
                  <h3 className="font-medium text-sm">
                    {currentInsight.title}
                  </h3>
                </div>
                <div className="flex items-center gap-2">
                  {currentInsight.blockchain && (
                    <Badge variant="outline" className="text-xs">
                      {currentInsight.blockchain === 'ethereum' ? 'ETH' : 'SOL'}
                    </Badge>
                  )}
                  <Badge 
                    variant={currentInsight.priority === 'high' ? 'destructive' : 'secondary'} 
                    className="text-xs"
                  >
                    {currentInsight.priority}
                  </Badge>
                </div>
              </div>
              
              <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
                {currentInsight.description}
              </p>
              
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-xs text-gray-500">Confidence:</span>
                  <Progress value={currentInsight.confidence} className="w-16 h-2" />
                  <span className="text-xs font-medium">{currentInsight.confidence}%</span>
                </div>
                
                {currentInsight.action && (
                  <Button 
                    size="sm" 
                    variant="outline" 
                    onClick={currentInsight.action.onClick}
                    className="text-xs"
                  >
                    {currentInsight.action.label}
                  </Button>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Insight Navigation */}
        {insights.length > 1 && (
          <div className="flex justify-center gap-2 pt-2">
            {insights.map((_, index) => (
              <button
                key={index}
                onClick={() => setCurrentInsightIndex(index)}
                className={cn(
                  "w-2 h-2 rounded-full transition-colors",
                  index === currentInsightIndex 
                    ? "bg-purple-600 dark:bg-purple-400" 
                    : "bg-gray-300 dark:bg-gray-600"
                )}
              />
            ))}
          </div>
        )}

        {/* Last Analysis Time */}
        {stats.lastAnalysisTime && (
          <div className="pt-2 text-center">
            <p className="text-xs text-gray-500 flex items-center justify-center gap-1">
              <Clock className="h-3 w-3" />
              Last analysis: {stats.lastAnalysisTime.toLocaleTimeString()}
            </p>
          </div>
        )}
      </CardContent>

      {/* Animated Background */}
      <div className="absolute inset-0 pointer-events-none">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: [0, 0.1, 0] }}
          transition={{ duration: 3, repeat: Infinity }}
          className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-purple-500/10 to-transparent rounded-full"
        />
      </div>
    </Card>
  );
};