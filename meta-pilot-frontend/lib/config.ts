/**
 * MetaPilot Configuration
 * Centralized configuration for Solana programs and AI integration
 */

export const SOLANA_CONFIG = {
  // Network configuration
  NETWORK: (process.env.NEXT_PUBLIC_SOLANA_NETWORK as 'localnet' | 'devnet' | 'mainnet-beta') || 'devnet',
  RPC_URL: process.env.NEXT_PUBLIC_SOLANA_RPC_URL || 'https://api.devnet.solana.com',
  COMMITMENT: 'confirmed' as const,
  
  // Program IDs
  PROGRAM_IDS: {
    DAO_VOTING: process.env.NEXT_PUBLIC_DAO_VOTING_PROGRAM_ID || 'CSxir1KkCbcKZpmUuEouTL4uLzwL4C7MTYyZrkxcKjv1',
    TOKEN_AUTOMATION: process.env.NEXT_PUBLIC_TOKEN_AUTOMATION_PROGRAM_ID || 'C7QmRHfSMg9YtcFyjGdfqA1PxBjZzefxxsBNggFTgcqR',
    YIELD_OPTIMIZER: process.env.NEXT_PUBLIC_YIELD_OPTIMIZER_PROGRAM_ID || 'uZWfyq9bwsiHUhLNXZKXEHkkbCpiin7Y3HvYNHDX4WQ',
    TRANSACTION_SCHEDULER: process.env.NEXT_PUBLIC_TRANSACTION_SCHEDULER_PROGRAM_ID || 'E4vpdUcGQd9bnzePzwJPKFNxrA5WARgFVzm77zfh2Hpe',
  },
} as const;

export const AI_CONFIG = {
  ENABLED: process.env.NEXT_PUBLIC_AI_ENGINE_ENABLED === 'true',
  CONFIDENCE_THRESHOLD: parseInt(process.env.NEXT_PUBLIC_AI_CONFIDENCE_THRESHOLD || '70'),
  MAX_RETRIES: 3,
  TIMEOUT_MS: 30000,
} as const;

export const FEATURE_FLAGS = {
  DAO_VOTING: process.env.NEXT_PUBLIC_ENABLE_DAO_VOTING === 'true',
  TOKEN_AUTOMATION: process.env.NEXT_PUBLIC_ENABLE_TOKEN_AUTOMATION === 'true',
  YIELD_OPTIMIZATION: process.env.NEXT_PUBLIC_ENABLE_YIELD_OPTIMIZATION === 'true',
  TRANSACTION_SCHEDULING: process.env.NEXT_PUBLIC_ENABLE_TRANSACTION_SCHEDULING === 'true',
  MULTI_CHAIN: process.env.NEXT_PUBLIC_ENABLE_MULTI_CHAIN === 'true',
} as const;

export const WALLET_CONFIG = {
  PREFERRED: process.env.NEXT_PUBLIC_PREFERRED_WALLET || 'phantom',
  AUTO_CONNECT: true,
  MULTI_CHAIN_ENABLED: FEATURE_FLAGS.MULTI_CHAIN,
} as const;

export const DEBUG_CONFIG = {
  ENABLED: process.env.NEXT_PUBLIC_DEBUG_MODE === 'true',
  CONSOLE_LOGS: process.env.NEXT_PUBLIC_ENABLE_CONSOLE_LOGS === 'true',
  VERBOSE: process.env.NODE_ENV === 'development',
} as const;

// Validation functions
export function validateConfig() {
  const errors: string[] = [];

  // Validate required environment variables
  if (!SOLANA_CONFIG.RPC_URL) {
    errors.push('NEXT_PUBLIC_SOLANA_RPC_URL is required');
  }

  // Validate program IDs format (should be valid base58)
  Object.entries(SOLANA_CONFIG.PROGRAM_IDS).forEach(([name, id]) => {
    if (!id || id.length !== 44) {
      errors.push(`Invalid program ID for ${name}: ${id}`);
    }
  });

  // Validate AI confidence threshold
  if (AI_CONFIG.CONFIDENCE_THRESHOLD < 0 || AI_CONFIG.CONFIDENCE_THRESHOLD > 100) {
    errors.push('AI confidence threshold must be between 0 and 100');
  }

  if (errors.length > 0) {
    throw new Error(`Configuration validation failed:\n${errors.join('\n')}`);
  }

  return true;
}

// Network-specific configurations
export const NETWORK_CONFIGS = {
  localnet: {
    rpcUrl: 'http://127.0.0.1:8899',
    name: 'Local Validator',
    explorer: 'http://localhost:3000', // Local explorer if running
  },
  devnet: {
    rpcUrl: 'https://api.devnet.solana.com',
    name: 'Devnet',
    explorer: 'https://explorer.solana.com',
  },
  'mainnet-beta': {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    name: 'Mainnet Beta',
    explorer: 'https://explorer.solana.com',
  },
} as const;

// Get current network configuration
export function getCurrentNetworkConfig() {
  return NETWORK_CONFIGS[SOLANA_CONFIG.NETWORK];
}

// Export types for TypeScript support
export type SolanaNetwork = keyof typeof NETWORK_CONFIGS;
export type ProgramName = keyof typeof SOLANA_CONFIG.PROGRAM_IDS;
