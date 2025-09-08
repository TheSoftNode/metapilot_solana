"use client";

import React from "react";
import { Wallet } from "lucide-react";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import UserDropdown from "@/components/Shared/UserDropdown";
import { useMultiChainWallet } from "@/hooks/use-multi-chain-wallet";
import { useWeb3AuthConnect } from "@web3auth/modal/react";

interface WalletConnectButtonProps {
    buttonClass?: string;
    className?: string;
}

const WalletConnectButton: React.FC<WalletConnectButtonProps> = ({ buttonClass, className }) => {
    const { isConnected } = useMultiChainWallet();
    const { connect } = useWeb3AuthConnect();
    
    const handleConnect = async () => {
        try {
            await connect();
        } catch (error) {
            console.error("Web3Auth connection failed:", error);
        }
    };

    return (
        <>
            {isConnected ? (
                <UserDropdown className={className} />
            ) : (
                <motion.div
                    whileHover={{ scale: 1.03 }}
                    whileTap={{ scale: 0.97 }}
                    className={cn("inline-flex", className)}
                >
                    <Button
                        onClick={handleConnect}
                        className={`bg-blue-600 text-white font-medium py-3 px-8 ${buttonClass} rounded-full hover:bg-blue-700 transition-colors`}
                    >
                        <Wallet className="mr-2 h-4 w-4" /> Connect Wallet
                    </Button>
                </motion.div>
            )}
        </>
    );
};

export default WalletConnectButton;

