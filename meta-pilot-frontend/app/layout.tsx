import type { Metadata } from "next";
import localFont from "next/font/local";
import "./globals.css";
import { ThemeProvider } from "@/components/Providers/theme-provider";
import ChatSupport from "@/components/Shared/ChatSupport";
import { Web3AuthProviderComponent } from "@/components/Providers/web3auth-provider";

const geistSans = localFont({
  src: "./fonts/GeistVF.woff",
  variable: "--font-geist-sans",
  weight: "100 900",
});
const geistMono = localFont({
  src: "./fonts/GeistMonoVF.woff",
  variable: "--font-geist-mono",
  weight: "100 900",
});

export const metadata: Metadata = {
  title: "MetaPilot | Web3 Automation Platform",
  description: "Automate your Web3 tasks - DAO voting, gas optimization, NFT purchases, token swaps, and more. Let AI handle your DeFi activities.",
  keywords: ["Web3", "automation", "DAO voting", "gas optimization", "NFT", "DeFi", "MetaMask", "blockchain", "Ethereum", "smart contracts"],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <Web3AuthProviderComponent>
          <ThemeProvider
            attribute="class"
            defaultTheme="system"
            enableSystem
            disableTransitionOnChange
          >
            {children}
            <ChatSupport />
          </ThemeProvider>
        </Web3AuthProviderComponent>
      </body>
    </html>
  );
}
