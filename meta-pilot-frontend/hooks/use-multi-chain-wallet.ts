"use client";

import { useWeb3AuthConnect, useWeb3AuthDisconnect, useWeb3AuthUser, useWeb3Auth } from "@web3auth/modal/react";
import { useAccount } from "wagmi"; // Ethereum
import { useSolanaWallet } from "@web3auth/modal/react/solana"; // Solana
import { WALLET_CONNECTORS, AUTH_CONNECTION } from "@web3auth/modal";
import { getED25519Key } from "@web3auth/modal";
import { Keypair, Connection } from "@solana/web3.js";
import { useEffect, useState } from "react";

export interface MultiChainWalletState {
    ethereum: {
        address?: string;
        isConnected: boolean;
        balance?: string;
        symbol?: string;
        chainId?: number;
    };
    solana: {
        address?: string;
        isConnected: boolean;
        balance?: number;
        symbol: string;
        connection?: Connection;
    };
    isLoading: boolean;
    error?: string;
}

export function useMultiChainWallet() {
    // Web3Auth hooks
    const { connectTo, isConnected, loading: connectLoading, error: connectError } = useWeb3AuthConnect();
    const { disconnect, loading: disconnectLoading } = useWeb3AuthDisconnect();
    const { userInfo } = useWeb3AuthUser();
    const { provider, isInitialized } = useWeb3Auth();
    
    // Ethereum wallet (Wagmi)
    const { address: ethAddress, isConnected: ethConnected } = useAccount();
    
    // Solana wallet
    const { accounts: solAccounts, connection: solConnection } = useSolanaWallet();
    
    // Local state
    const [solanaAddress, setSolanaAddress] = useState<string | undefined>();
    const [solanaBalance] = useState<number>(0);
    const [ethereumBalance] = useState<string>("0");
    const [currentNetwork, setCurrentNetwork] = useState<'ethereum' | 'solana'>('ethereum');
    
    // Derive Solana address from Ethereum private key
    useEffect(() => {
        const deriveSolanaAddress = async () => {
            if (provider && isConnected) {
                try {
                    const ethPrivateKey = await provider.request({ method: "private_key" });
                    const privateKey = getED25519Key(ethPrivateKey as string).sk.toString("hex");
                    const secretKey = new Uint8Array(
                        privateKey.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []
                    );
                    const keypair = Keypair.fromSecretKey(secretKey);
                    setSolanaAddress(keypair.publicKey.toBase58());
                } catch (error) {
                    console.error("Error deriving Solana address:", error);
                }
            }
        };
        
        deriveSolanaAddress();
    }, [provider, isConnected]);

    // Social authentication functions for custom modal
    const handleSocialAuth = async (provider: string) => {
        try {
            // Check if Web3Auth is initialized before attempting connection
            if (!isInitialized) {
                throw new Error("Web3Auth is not initialized yet. Please wait a moment and try again.");
            }

            // Map provider names to AUTH_CONNECTION constants
            const providerMap: Record<string, any> = {
                google: AUTH_CONNECTION.GOOGLE,
                facebook: AUTH_CONNECTION.FACEBOOK,
                twitter: AUTH_CONNECTION.TWITTER,
                discord: AUTH_CONNECTION.DISCORD,
            };

            const authConnection = providerMap[provider];
            if (!authConnection) {
                throw new Error(`Unsupported authentication provider: ${provider}`);
            }

            await connectTo(WALLET_CONNECTORS.AUTH, {
                authConnection: authConnection,
                authConnectionId: `w3a-${provider}-demo`,
            });
        } catch (error) {
            console.error(`${provider} authentication failed:`, error);
            throw error;
        }
    };


    // Traditional wallet connection
    const handleWalletConnect = async (walletId: string) => {
        try {
            // Check if Web3Auth is initialized before attempting connection
            if (!isInitialized) {
                throw new Error("Web3Auth is not initialized yet. Please wait a moment and try again.");
            }

            // Only supported Ethereum wallets (verified to work)
            const walletConnectorMap: Record<string, string> = {
                metamask: WALLET_CONNECTORS.METAMASK,
                walletconnect: WALLET_CONNECTORS.WALLET_CONNECT_V2,
                coinbase: WALLET_CONNECTORS.COINBASE,
            };

            const connector = walletConnectorMap[walletId];
            if (!connector) {
                throw new Error(`Unsupported wallet: ${walletId}`);
            }

            await connectTo(connector as any);
        } catch (error) {
            console.error(`${walletId} connection failed:`, error);
            throw error;
        }
    };

    // Format address for display
    const formatAddress = (addr?: string) => {
        if (!addr) return "";
        return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
    };

    // Build wallet state object
    const walletState: MultiChainWalletState = {
        ethereum: {
            address: ethAddress,
            isConnected: ethConnected,
            balance: ethereumBalance,
            symbol: "ETH",
            chainId: 1,
        },
        solana: {
            address: solanaAddress || solAccounts?.[0],
            isConnected: !!solanaAddress || !!solAccounts?.[0],
            balance: solanaBalance,
            symbol: "SOL",
            connection: solConnection || undefined,
        },
        isLoading: connectLoading || disconnectLoading || !isInitialized,
        error: connectError?.message,
    };

    return {
        // Connection state
        isConnected,
        isConnecting: connectLoading || disconnectLoading,
        isInitialized,
        error: connectError,
        
        // User information
        userInfo,
        
        // Wallet state
        ...walletState,
        
        // Connection methods for custom modal
        handleSocialAuth,
        handleWalletConnect,
        disconnect,
        
        // Utility
        formatAddress,
        
        // Network management
        currentNetwork,
        setCurrentNetwork,
        
        // Legacy compatibility (for existing components)
        address: ethAddress || solanaAddress || solAccounts?.[0],
        status: isConnected ? 'connected' : 'disconnected',
        balance: { formatted: ethereumBalance, symbol: "ETH" },
        currentWallet: ethAddress ? 'ethereum' : (solanaAddress || solAccounts?.[0]) ? 'solana' : 'none',
        primaryAddress: ethAddress || solanaAddress || solAccounts?.[0],
        
        // Connection methods (legacy)
        connect: handleSocialAuth,
    };
}