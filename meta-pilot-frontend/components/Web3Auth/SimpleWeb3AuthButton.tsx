"use client";

import { useWeb3AuthConnect } from "@web3auth/modal/react";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

interface SimpleWeb3AuthButtonProps {
    onSuccess?: () => void;
}

export const SimpleWeb3AuthButton: React.FC<SimpleWeb3AuthButtonProps> = ({ onSuccess }) => {
    const { connect, isConnected, loading, error } = useWeb3AuthConnect();

    const handleConnect = async () => {
        try {
            console.log("Attempting to connect with Web3Auth...");
            if (connect) {
                await connect();
                console.log("Web3Auth connection successful");
                onSuccess?.();
            } else {
                console.error("Web3Auth connect function not available");
            }
        } catch (error) {
            console.error("Web3Auth connection failed:", error);
        }
    };

    if (isConnected) {
        return (
            <Button disabled className="bg-green-500">
                Connected ✓
            </Button>
        );
    }

    return (
        <Button 
            onClick={handleConnect} 
            disabled={loading}
            className="w-full bg-blue-600 hover:bg-blue-700 text-white"
        >
            {loading ? (
                <>
                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                    Connecting...
                </>
            ) : (
                "Connect with Web3Auth"
            )}
            {error && (
                <div className="text-red-500 text-xs mt-2">
                    Error: {error.message}
                </div>
            )}
        </Button>
    );
};