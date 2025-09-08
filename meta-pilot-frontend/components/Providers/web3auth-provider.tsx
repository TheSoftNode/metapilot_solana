"use client";

import { Web3AuthProvider, type Web3AuthContextConfig } from "@web3auth/modal/react";
import { WagmiProvider } from "@web3auth/modal/react/wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { WEB3AUTH_NETWORK, WALLET_CONNECTORS } from "@web3auth/modal";
import { ReactNode } from "react";

const clientId = process.env.NEXT_PUBLIC_WEB3AUTH_CLIENT_ID || "BIk6yVE73Q6MR53UPuPIMuq3gjc6ji65nsx9Zhklz2qsUZx7yMh_WXWgEFvOhcJJpaKxuK0nBVXf4rWlRB986to";
const queryClient = new QueryClient();

// Web3Auth configuration - minimal setup
const web3AuthContextConfig: Web3AuthContextConfig = {
  web3AuthOptions: {
    clientId,
    web3AuthNetwork: WEB3AUTH_NETWORK.SAPPHIRE_DEVNET,
    uiConfig: {
        appName: "MetaPilot - AI Web3 Automation",
        mode: "dark", 
        logoLight: "/metapilot-logo.png",
        logoDark: "/metapilot-logo.png",
        defaultLanguage: "en",
        theme: {
          primary: "#3B82F6",
        },
      },
  }
};

export function Web3AuthProviderComponent({ 
  children 
}: {
  children: ReactNode
}) {
  return (
    <Web3AuthProvider config={web3AuthContextConfig}>
      <QueryClientProvider client={queryClient}>
        <WagmiProvider>
          {children}
        </WagmiProvider>
      </QueryClientProvider>
    </Web3AuthProvider>
  );
}

// Export the config for use in other components
export { web3AuthContextConfig };