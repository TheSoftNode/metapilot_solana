# MetaPilot Solana Smart Contracts

This directory contains the MetaPilot smart contracts for Solana blockchain, built with the Anchor framework.

## 🏗️ Architecture

### Programs Overview

1. **DAO Voting** (`dao-voting`) - Automated proposal analysis and voting
2. **Token Automation** (`token-automation`) - AI-optimized token swaps and trading
3. **Yield Optimizer** (`yield-optimizer`) - Cross-protocol yield farming optimization
4. **Transaction Scheduler** (`transaction-scheduler`) - Time-based transaction execution

### Directory Structure

```
metapilot-programs/metapilot-core/
├── Anchor.toml                 # Anchor workspace configuration
├── Cargo.toml                  # Rust workspace configuration
├── deploy.sh                   # Deployment script
├── programs/                   # Smart contract programs
│   ├── dao-voting/
│   ├── token-automation/
│   ├── yield-optimizer/
│   └── transaction-scheduler/
└── target/                     # Build artifacts
```

## 🚀 Quick Start

### Prerequisites

1. **Rust & Cargo** - [Install Rust](https://rustup.rs/)
2. **Solana CLI** - [Install Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
3. **Anchor Framework** - [Install Anchor](https://www.anchor-lang.com/docs/installation)

```bash
# Verify installations
rust --version
solana --version
anchor --version
```

### Build & Deploy

1. **Navigate to the project directory:**

```bash
cd metapilot-programs/metapilot-core
```

2. **Build the programs:**

```bash
# Using cargo (recommended - bypasses IDL issues)
cargo build-sbf

# Or using anchor (if IDL generation is fixed)
anchor build
```

3. **Deploy to Solana:**

```bash
# Deploy to devnet (recommended for testing)
./deploy.sh devnet

# Deploy to localnet (for local development)
./deploy.sh localnet

# Deploy to mainnet (production only)
./deploy.sh mainnet-beta
```

## 📋 Program Details

### DAO Voting Program

**Purpose**: Automated governance participation with AI-powered proposal analysis

**Key Features**:

- Create and manage proposals
- Automated voting based on user-defined rules
- AI sentiment analysis of proposals
- Gas-optimized execution

**Main Instructions**:

- `create_proposal` - Create new governance proposal
- `cast_vote` - Vote on existing proposal
- `execute_proposal` - Execute passed proposal
- `close_proposal` - Close expired proposal

### Token Automation Program

**Purpose**: Intelligent token trading and swap automation

**Key Features**:

- Automated token swaps based on price thresholds
- Multi-DEX routing for best prices
- Slippage protection
- AI-optimized timing

**Main Instructions**:

- `create_swap_config` - Configure automated swap parameters
- `execute_swap` - Execute token swap
- `update_price_threshold` - Modify price triggers
- `pause_automation` - Temporarily disable automation

### Yield Optimizer Program

**Purpose**: Cross-protocol yield farming optimization

**Key Features**:

- Multiple yield strategy types (conservative, balanced, aggressive)
- Automated rebalancing based on market conditions
- Risk management and portfolio optimization
- Reward harvesting automation

**Main Instructions**:

- `create_strategy` - Initialize yield farming strategy
- `deposit` - Deposit funds into strategy
- `withdraw` - Withdraw funds from strategy
- `rebalance` - Rebalance portfolio allocations
- `harvest_rewards` - Claim farming rewards

### Transaction Scheduler Program

**Purpose**: Time-based transaction execution with gas optimization

**Key Features**:

- Schedule transactions for future execution
- Gas price optimization
- Conditional execution based on market conditions
- Batch transaction processing

**Main Instructions**:

- `schedule_transaction` - Schedule future transaction
- `execute_scheduled` - Execute scheduled transaction
- `cancel_scheduled` - Cancel pending transaction
- `update_schedule` - Modify scheduled transaction

## 🔧 Configuration

### Environment Variables

The deployment script automatically generates environment configuration for the frontend:

```bash
# Solana Network Configuration
NEXT_PUBLIC_SOLANA_NETWORK=devnet
NEXT_PUBLIC_SOLANA_RPC_URL=https://api.devnet.solana.com

# Program IDs (auto-generated after deployment)
NEXT_PUBLIC_DAO_VOTING_PROGRAM_ID=CSxir1KkCbcKZpmUuEouTL4uLzwL4C7MTYyZrkxcKjv1
NEXT_PUBLIC_TOKEN_AUTOMATION_PROGRAM_ID=C7QmRHfSMg9YtcFyjGdfqA1PxBjZzefxxsBNggFTgcqR
NEXT_PUBLIC_YIELD_OPTIMIZER_PROGRAM_ID=uZWfyq9bwsiHUhLNXZKXEHkkbCpiin7Y3HvYNHDX4WQ
NEXT_PUBLIC_TRANSACTION_SCHEDULER_PROGRAM_ID=E4vpdUcGQd9bnzePzwJPKFNxrA5WARgFVzm77zfh2Hpe
```

### Network Configuration

```toml
# Anchor.toml
[programs.devnet]
dao_voting = "CSxir1KkCbcKZpmUuEouTL4uLzwL4C7MTYyZrkxcKjv1"
token_automation = "C7QmRHfSMg9YtcFyjGdfqA1PxBjZzefxxsBNggFTgcqR"
yield_optimizer = "uZWfyq9bwsiHUhLNXZKXEHkkbCpiin7Y3HvYNHDX4WQ"
transaction_scheduler = "E4vpdUcGQd9bnzePzwJPKFNxrA5WARgFVzm77zfh2Hpe"
```

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests for specific program
cargo test --manifest-path programs/dao-voting/Cargo.toml
```

### Integration Tests

```bash
# Run integration tests (requires deployed programs)
anchor test
```

### Manual Testing

Use the Solana CLI to interact with deployed programs:

```bash
# Check program account
solana account <PROGRAM_ID>

# Get program logs
solana logs <PROGRAM_ID>
```

## 🔗 Frontend Integration

### React Hooks

The smart contracts are integrated with the frontend through custom React hooks:

```typescript
import {
  useMetaPilotClient,
  useDAOProposals,
} from "@/hooks/use-solana-programs";

// Initialize client
const client = useMetaPilotClient();

// Use DAO proposals
const { proposals, createProposal, castVote } = useDAOProposals(client);
```

### AI Integration

Smart contracts work seamlessly with the MetaPilot AI engine:

```typescript
import { createAIIntegration } from "@/lib/ai-solana-integration";

// Create AI-powered automation
const aiIntegration = createAIIntegration(aiEngine, solanaClient, userWallet);

// Automated proposal voting
const result = await aiIntegration.analyzeAndVoteOnProposal(
  proposalId,
  userRules
);
```

## 🛠️ Development

### Local Development

1. **Start local Solana validator:**

```bash
solana-test-validator
```

2. **Deploy to local network:**

```bash
./deploy.sh localnet
```

3. **Update frontend configuration:**

```bash
cp ../meta-pilot-frontend/.env.local ../meta-pilot-frontend/.env.development
```

### IDL Alternative

Since we're using `cargo build-sbf` instead of Anchor's IDL generation, the frontend uses:

1. **Direct program interfaces** (`lib/solana-programs.ts`)
2. **Instruction builders** for each program
3. **Account derivation helpers** for PDA calculations
4. **Borsh serialization** for data structures

This approach provides:

- ✅ No dependency on anchor-syn
- ✅ Direct control over instruction data
- ✅ Better performance
- ✅ Production-ready deployment

## 📚 Resources

- [Anchor Framework Documentation](https://www.anchor-lang.com/)
- [Solana Program Library](https://spl.solana.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [MetaPilot AI Engine Documentation](../meta-pilot-ai-engine/README.md)

## 🚨 Important Notes

### Security Considerations

- All programs use PDA (Program Derived Addresses) for security
- Input validation and access control implemented
- Reentrancy protection in place
- Overflow/underflow protection

### Gas Optimization

- Programs optimized for minimal compute units
- Efficient account layout and packing
- Batch operations where possible
- Smart transaction timing

### Production Checklist

- [ ] Deploy to devnet and test thoroughly
- [ ] Security audit (recommended for mainnet)
- [ ] Update frontend environment variables
- [ ] Test AI integration end-to-end
- [ ] Monitor program performance
- [ ] Set up alerting for program errors

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
