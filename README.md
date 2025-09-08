# MetaPilot 🚀

**AI-Powered Web3 Automation Platform**

[![Next.js](https://img.shields.io/badge/Next.js-14-black?logo=next.js)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-blue?logo=typescript)](https://www.typescriptlang.org/)
[![Web3Auth](https://img.shields.io/badge/Web3Auth-Multi--Chain-purple)](https://web3auth.io/)
[![Solana](https://img.shields.io/badge/Solana-Blockchain-green?logo=solana)](https://solana.com/)
[![Ethereum](https://img.shields.io/badge/Ethereum-Compatible-blue?logo=ethereum)](https://ethereum.org/)

MetaPilot is a cutting-edge Web3 automation platform that leverages artificial intelligence to streamline blockchain interactions. Connect once with social login and get access to both Ethereum and Solana ecosystems with intelligent automation for DAO governance, token swaps, yield optimization, and payments.

## 🌟 Features

### 🔐 **Multi-Chain Authentication**
- **One-Click Social Login**: Google, Facebook, Twitter, Discord
- **Wallet Integration**: MetaMask, WalletConnect, Coinbase Wallet
- **Multi-Chain Access**: Single authentication provides both Ethereum and Solana wallets
- **SSR Support**: Server-side rendering with Next.js compatibility

### 🤖 **AI-Powered Automation**
- **Smart DAO Voting**: Automated governance participation based on user preferences
- **Market Analysis**: AI-driven token swap timing optimization
- **Gas Optimization**: Intelligent transaction timing for cost savings
- **Yield Strategies**: Automated portfolio rebalancing for maximum returns

### 🏛️ **DAO Governance**
- **Multi-Chain DAOs**: Support for both Ethereum and Solana governance
- **Popular DAOs**: Nouns, ENS, Compound, AAVE, Uniswap (Ethereum)
- **Solana Ecosystem**: Marinade, Mango Markets, Solend, Realms
- **Custom Rules**: Personalized voting automation

### 💰 **Solana Pay Integration**
- **QR Code Generation**: Create payment requests for any SPL token
- **Multi-Token Support**: SOL, USDC, USDT, and custom tokens
- **Mobile Wallet Compatible**: Works with all Solana Pay wallets
- **Instant Settlements**: Sub-second transaction confirmations

### 📊 **Real-Time Insights**
- **AI Dashboard**: Intelligent recommendations based on portfolio activity
- **Multi-Chain Monitoring**: Track assets across both networks
- **Performance Analytics**: Gas savings, automation success rates
- **Market Opportunities**: AI-identified profit opportunities

## 🛠️ Technology Stack

### **Frontend**
- **Next.js 14**: React framework with App Router
- **TypeScript**: Type-safe development
- **Tailwind CSS**: Utility-first styling
- **Framer Motion**: Smooth animations
- **Radix UI**: Accessible component primitives

### **Blockchain Integration**
- **Web3Auth**: Multi-chain authentication infrastructure
- **Wagmi**: Ethereum interactions
- **Solana Web3.js**: Solana blockchain integration
- **Solana Pay**: Payment protocol implementation

### **AI Engine**
- **Custom AI Client**: TypeScript-based decision engine
- **Mock Analytics**: Development-ready AI responses
- **Rule Engine**: User-configurable automation logic
- **Market Analysis**: Price and sentiment evaluation

### **State Management**
- **React Hooks**: Modern state management
- **Local Storage**: Persistent user preferences
- **Context API**: Global application state

## 🚀 Quick Start

### Prerequisites

```bash
Node.js >= 16.0.0
npm >= 7.0.0
Git
```

### Installation

1. **Clone the Repository**
```bash
git clone https://github.com/your-username/MetaPilot.git
cd MetaPilot/meta-pilot-frontend
```

2. **Install Dependencies**
```bash
npm install
```

3. **Environment Setup**
```bash
cp .env.example .env.local
```

4. **Configure Environment Variables**
```bash
# Web3Auth Configuration
NEXT_PUBLIC_WEB3AUTH_CLIENT_ID="your_web3auth_client_id"

# AI Engine (Optional)
NEXT_PUBLIC_AI_ENGINE_URL="http://localhost:3001"
NEXT_PUBLIC_AI_ENGINE_API_KEY="your_ai_api_key"
```

5. **Start Development Server**
```bash
npm run dev
```

6. **Open Application**
Navigate to [http://localhost:3000](http://localhost:3000)

## 🔧 Configuration

### Web3Auth Setup

1. **Create Web3Auth Account**
   - Visit [Web3Auth Dashboard](https://dashboard.web3auth.io/)
   - Create a new project
   - Select "Web" platform

2. **Configure Authentication**
   - Enable Google, Facebook, Twitter, Discord
   - Set redirect URLs: `http://localhost:3000`
   - Copy Client ID to `.env.local`

3. **Network Configuration**
   - Environment: Sapphire Devnet (development)
   - Chains: Ethereum, Solana
   - Enable multi-chain mode

### Solana Configuration

The application automatically connects to Solana devnet. For mainnet deployment:

1. Update RPC endpoints in configuration
2. Update token mint addresses
3. Test with small amounts first

## 📱 Usage Guide

### Getting Started

1. **Connect Wallet**
   - Click "Connect Wallet" 
   - Choose social login or wallet connection
   - Approve permissions

2. **Multi-Chain Access**
   - Ethereum wallet: Automatic via Web3Auth
   - Solana wallet: Derived from same authentication
   - Switch between chains seamlessly

### DAO Automation

1. **Navigate to Tasks**
   - Go to Dashboard → Tasks
   - Click "Create Task"
   - Select "DAO Voting"

2. **Configure Automation**
   - Choose your preferred DAOs
   - Set voting rules and preferences
   - Enable automation

3. **Monitor Activity**
   - View active automations
   - Check voting history
   - Adjust rules as needed

### Solana Pay

1. **Access Solana Pay**
   - Go to Dashboard → Wallet → Solana Pay tab
   - Enter payment details
   - Generate QR code

2. **Payment Options**
   - SOL, USDC, USDT support
   - Custom SPL tokens
   - Mobile wallet scanning

### AI Insights

- **Dashboard Overview**: Real-time recommendations
- **Market Alerts**: AI-identified opportunities
- **Performance Tracking**: Automation success rates
- **Gas Optimization**: Cost-saving suggestions

## 🏗️ Project Structure

```
meta-pilot-frontend/
├── app/                          # Next.js App Router
│   ├── dashboard/               # Dashboard pages
│   │   ├── page.tsx            # Main dashboard
│   │   ├── tasks/              # Task management
│   │   └── wallet/             # Wallet interface
│   ├── globals.css             # Global styles
│   └── layout.tsx              # Root layout
├── components/                  # React components
│   ├── Dashbaord/              # Dashboard components
│   │   ├── AIInsights.tsx      # AI recommendation panel
│   │   ├── DashboardOverview.tsx # Main dashboard
│   │   └── Tasks/              # Task management UI
│   ├── Providers/              # Context providers
│   │   └── web3auth-provider.tsx # Web3Auth setup
│   ├── Shared/                 # Reusable components
│   ├── Solana/                 # Solana-specific components
│   │   └── SolanaPayQR.tsx     # Payment QR generation
│   ├── ui/                     # UI primitives
│   └── Web3Auth/               # Authentication components
│       └── Web3AuthModal.tsx   # Login modal
├── hooks/                      # React hooks
│   ├── use-ai-automation.ts    # AI automation logic
│   ├── use-multi-chain-wallet.ts # Multi-chain wallet
│   └── use-wallet.ts           # Wallet utilities
├── lib/                        # Utility libraries
│   ├── ai-client.ts           # AI engine integration
│   ├── ai-solana-integration.ts # Solana AI bridge
│   └── utils.ts               # Helper functions
└── public/                     # Static assets
```

## 🔌 API Reference

### AI Client

```typescript
import { aiClient } from '@/lib/ai-client';

// Analyze DAO proposal
const analysis = await aiClient.analyzeProposal({
  proposalId: "123",
  daoName: "Example DAO",
  proposalText: "Proposal description...",
  proposalType: "governance"
}, userRules, context);

// Market analysis
const market = await aiClient.analyzeMarket({
  tokenA: "SOL",
  tokenB: "USDC",
  currentPrice: 100,
  // ... other parameters
}, userRules);
```

### Multi-Chain Wallet

```typescript
import { useMultiChainWallet } from '@/hooks/use-multi-chain-wallet';

function MyComponent() {
  const { 
    ethereum, 
    solana, 
    handleSocialAuth, 
    handleWalletConnect,
    isConnected 
  } = useMultiChainWallet();

  // Ethereum wallet: ethereum.address
  // Solana wallet: solana.address
}
```

### Solana Pay

```typescript
import { SolanaPayQR } from '@/components/Solana/SolanaPayQR';

<SolanaPayQR 
  defaultRecipient="wallet_address"
  onPaymentRequest={(request) => {
    console.log('Payment created:', request);
  }}
/>
```

## 🧪 Testing

### Unit Tests
```bash
npm run test
```

### Type Checking
```bash
npm run type-check
```

### Linting
```bash
npm run lint
npm run lint:fix
```

### Build Verification
```bash
npm run build
```

## 🚀 Deployment

### Vercel (Recommended)

1. **Connect Repository**
   - Import project to Vercel
   - Connect GitHub repository

2. **Environment Variables**
   - Add production Web3Auth Client ID
   - Configure AI engine endpoints
   - Set domain redirects

3. **Deploy**
   ```bash
   vercel --prod
   ```

### Docker

```bash
# Build image
docker build -t metapilot-frontend .

# Run container
docker run -p 3000:3000 metapilot-frontend
```

### Manual Deployment

```bash
# Build application
npm run build

# Start production server
npm start
```

## 🔐 Security

### Authentication Security
- **Web3Auth**: Enterprise-grade authentication
- **Multi-Factor**: Social + wallet verification
- **Key Management**: Non-custodial key derivation
- **Session Management**: Secure token handling

### Smart Contract Safety
- **Read-Only Operations**: No direct contract modifications
- **User Approval**: All transactions require explicit consent
- **Network Validation**: Automatic chain verification
- **Error Handling**: Comprehensive failure recovery

### Data Privacy
- **Local Storage**: Sensitive data kept client-side
- **No PII Collection**: Minimal user data retention
- **Encrypted Communications**: HTTPS/WSS only
- **Audit Trail**: Transaction logging for transparency

## 🤝 Contributing

### Development Workflow

1. **Fork Repository**
2. **Create Feature Branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make Changes**
   - Follow TypeScript best practices
   - Add tests for new functionality
   - Update documentation

4. **Commit Changes**
   ```bash
   git commit -m "Add amazing feature"
   ```

5. **Create Pull Request**
   - Describe changes thoroughly
   - Include screenshots for UI changes
   - Reference related issues

### Code Standards

- **TypeScript**: Strict mode enabled
- **ESLint**: Airbnb configuration
- **Prettier**: Code formatting
- **Conventional Commits**: Semantic commit messages

### Testing Requirements
- Unit tests for new functions
- Integration tests for API changes
- Manual testing for UI components

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Web3Auth Team**: For excellent multi-chain authentication
- **Solana Foundation**: For the innovative payment protocol
- **Vercel**: For seamless deployment platform
- **Next.js Team**: For the outstanding React framework
- **Open Source Community**: For countless valuable libraries

## 📞 Support

### Documentation
- **Web3Auth Docs**: https://web3auth.io/docs
- **Solana Pay Guide**: https://docs.solanapay.com
- **Next.js Documentation**: https://nextjs.org/docs

### Community
- **Discord**: [Join our server](https://discord.gg/metapilot)
- **Twitter**: [@MetaPilotHQ](https://twitter.com/metapilothq)
- **GitHub Issues**: Report bugs and feature requests

### Contact
- **Email**: support@metapilot.io
- **Website**: https://metapilot.io
- **Documentation**: https://docs.metapilot.io

---

**Built with ❤️ for the Web3 community**

*MetaPilot - Automating the future of decentralized finance*
