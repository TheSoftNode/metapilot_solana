#!/bin/bash

# MetaPilot Professional Deployment Script
# Supports: localnet, devnet, testnet, mainnet-beta
# Features: Structured tracking, skip re-deployment, professional logging

set -e

SCRIPT_VERSION="1.0.0"
SCRIPT_NAME="MetaPilot Professional Deployment"

echo "🚀 $SCRIPT_NAME v$SCRIPT_VERSION"
echo "================================================="

# Configuration
NETWORK=${1:-devnet}
KEYPAIR_PATH=${2:-keys/deployer.json}
FORCE_REDEPLOY=${3:-false}

# Network validation
VALID_NETWORKS=("localnet" "devnet" "testnet" "mainnet-beta")
if [[ ! " ${VALID_NETWORKS[@]} " =~ " ${NETWORK} " ]]; then
    echo "❌ Invalid network: $NETWORK"
    echo "Valid options: localnet, devnet, testnet, mainnet-beta"
    echo ""
    echo "Usage: ./metapilot-deploy.sh <network> [keypair_path] [force_redeploy]"
    echo "Examples:"
    echo "  ./metapilot-deploy.sh devnet"
    echo "  ./metapilot-deploy.sh localnet keys/my-key.json"
    echo "  ./metapilot-deploy.sh mainnet-beta keys/mainnet.json true"
    exit 1
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
deploy() { echo -e "${PURPLE}🚀 $1${NC}"; }
info() { echo -e "${CYAN}📋 $1${NC}"; }

# Create network-specific directories
DEPLOYMENT_ROOT="deployments"
NETWORK_DIR="$DEPLOYMENT_ROOT/$NETWORK"
STATE_FILE="$NETWORK_DIR/deployment-state.json"
ENV_FILE="$NETWORK_DIR/program-ids.env"
SUMMARY_FILE="$NETWORK_DIR/deployment-summary.md"

mkdir -p "$NETWORK_DIR"

log "Network: $NETWORK"
log "Keypair: $KEYPAIR_PATH"
log "Deployment directory: $NETWORK_DIR"
log "Force redeploy: $FORCE_REDEPLOY"

# Initialize state file
if [ ! -f "$STATE_FILE" ] || [ "$FORCE_REDEPLOY" = "true" ]; then
    cat > "$STATE_FILE" << EOF
{
  "network": "$NETWORK",
  "deployment_version": "$SCRIPT_VERSION",
  "programs": {},
  "metadata": {
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "deployer_keypair": "$KEYPAIR_PATH",
    "total_programs": 4,
    "successful_deployments": 0,
    "failed_deployments": 0
  }
}
EOF
    log "Initialized deployment state for $NETWORK"
fi

# Network configuration
configure_network() {
    case "$NETWORK" in
        "localnet")
            log "Configuring for local validator"
            solana config set --url localhost
            RPC_URL="http://127.0.0.1:8899"
            MIN_BALANCE=1000000000  # 1 SOL
            ;;
        "devnet")
            log "Configuring for Devnet"
            solana config set --url devnet
            RPC_URL="https://api.devnet.solana.com"
            MIN_BALANCE=5000000000  # 5 SOL
            ;;
        "testnet")
            log "Configuring for Testnet"
            solana config set --url testnet
            RPC_URL="https://api.testnet.solana.com"
            MIN_BALANCE=10000000000  # 10 SOL
            ;;
        "mainnet-beta")
            log "Configuring for MAINNET"
            warning "YOU ARE DEPLOYING TO MAINNET!"
            echo -e "${RED}This will cost real SOL and deploy to production!${NC}"
            echo "Type 'DEPLOY_TO_MAINNET' to confirm:"
            read -r confirmation
            if [ "$confirmation" != "DEPLOY_TO_MAINNET" ]; then
                error "Mainnet deployment cancelled"
            fi
            solana config set --url mainnet-beta
            RPC_URL="https://api.mainnet-beta.solana.com"
            MIN_BALANCE=20000000000  # 20 SOL
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    for cmd in solana solana-keygen jq; do
        if ! command -v $cmd &> /dev/null; then
            error "$cmd not found. Please install it first."
        fi
    done
    
    # Check if we're in the right directory
    if [ ! -f "Anchor.toml" ]; then
        error "Anchor.toml not found. Please run from metapilot-core directory."
    fi
    
    success "All prerequisites met"
}

# Setup keypair
setup_keypair() {
    if [ ! -f "$KEYPAIR_PATH" ]; then
        log "Creating new keypair: $KEYPAIR_PATH"
        mkdir -p "$(dirname "$KEYPAIR_PATH")"
        solana-keygen new -o "$KEYPAIR_PATH" --no-passphrase --force
    fi
    
    solana config set --keypair "$KEYPAIR_PATH"
    
    # Get wallet info
    WALLET_ADDRESS=$(solana-keygen pubkey "$KEYPAIR_PATH")
    log "Wallet address: $WALLET_ADDRESS"
    
    # Check balance
    BALANCE=$(solana balance --lamports 2>/dev/null || echo "0")
    BALANCE_SOL=$(echo "scale=2; $BALANCE / 1000000000" | bc -l 2>/dev/null || echo "0")
    log "Wallet balance: $BALANCE_SOL SOL ($BALANCE lamports)"
    
    # Request airdrop if needed (non-mainnet only)
    if [ "$NETWORK" != "mainnet-beta" ] && [ "$BALANCE" -lt "$MIN_BALANCE" ]; then
        warning "Balance too low. Minimum required: $(echo "scale=2; $MIN_BALANCE / 1000000000" | bc -l) SOL"
        if [ "$NETWORK" = "localnet" ]; then
            success "Local validator provides sufficient balance automatically"
        else
            log "Attempting airdrop..."
            solana airdrop 2 2>/dev/null || warning "Airdrop failed - you may need to fund manually"
        fi
    fi
}

# Check if program is already deployed
is_deployed() {
    local program_name=$1
    if [ -f "$STATE_FILE" ]; then
        local status=$(jq -r ".programs[\"$program_name\"].status // \"not_found\"" "$STATE_FILE")
        if [ "$status" = "deployed" ] && [ "$FORCE_REDEPLOY" != "true" ]; then
            return 0
        fi
    fi
    return 1
}

# Save deployment result
save_deployment() {
    local program_name=$1
    local program_id=$2
    local status=$3
    local signature=$4
    local error_msg=$5
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    
    # Create temporary JSON for update
    local temp_file=$(mktemp)
    jq --arg name "$program_name" \
       --arg id "$program_id" \
       --arg status "$status" \
       --arg sig "$signature" \
       --arg error "$error_msg" \
       --arg time "$timestamp" \
       --arg deployer "$WALLET_ADDRESS" \
       '.programs[$name] = {
         "program_id": $id,
         "status": $status,
         "signature": $sig,
         "error": $error,
         "deployed_at": $time,
         "deployer": $deployer,
         "binary_path": ("target/deploy/" + ($name | gsub("-"; "_")) + ".so")
       } |
       .metadata.last_updated = $time |
       if $status == "deployed" then
         .metadata.successful_deployments += 1
       else
         .metadata.failed_deployments += 1
       end' "$STATE_FILE" > "$temp_file" && mv "$temp_file" "$STATE_FILE"
}

# Deploy individual program
deploy_program() {
    local program_name=$1
    local binary_name="${program_name//-/_}"
    local binary_path="target/deploy/${binary_name}.so"
    
    info "Processing $program_name..."
    
    # Check if already deployed
    if is_deployed "$program_name"; then
        local existing_id=$(jq -r ".programs[\"$program_name\"].program_id" "$STATE_FILE")
        success "$program_name already deployed: $existing_id"
        return 0
    fi
    
    # Verify binary exists
    if [ ! -f "$binary_path" ]; then
        error "Binary not found: $binary_path"
        save_deployment "$program_name" "null" "failed" "null" "Binary not found"
        return 1
    fi
    
    deploy "Deploying $program_name..."
    
    # Deploy with error handling
    local deploy_output
    if deploy_output=$(solana program deploy "$binary_path" --output json 2>&1); then
        local program_id=$(echo "$deploy_output" | jq -r '.programId // empty')
        local signature=$(echo "$deploy_output" | jq -r '.signature // empty')
        
        if [ -n "$program_id" ] && [ "$program_id" != "null" ]; then
            success "$program_name deployed successfully"
            success "Program ID: $program_id"
            log "Signature: $signature"
            
            # Verify deployment
            if solana account "$program_id" --output json >/dev/null 2>&1; then
                success "Deployment verified on-chain"
                save_deployment "$program_name" "$program_id" "deployed" "$signature" ""
                
                # Update Anchor.toml
                if [ -f "Anchor.toml" ]; then
                    if grep -q "^$program_name = " Anchor.toml; then
                        sed -i.bak "s/^$program_name = .*/$program_name = \"$program_id\"/" Anchor.toml
                    else
                        # Add to programs section
                        sed -i.bak "/\[programs\.$NETWORK\]/a\\
$program_name = \"$program_id\"" Anchor.toml
                    fi
                fi
                
                return 0
            else
                warning "Deployment verification failed"
                save_deployment "$program_name" "$program_id" "unverified" "$signature" "Verification failed"
                return 1
            fi
        else
            error "No program ID returned from deployment"
            save_deployment "$program_name" "null" "failed" "null" "No program ID returned"
            return 1
        fi
    else
        error "Deployment failed: $deploy_output"
        save_deployment "$program_name" "null" "failed" "null" "$deploy_output"
        return 1
    fi
}

# Generate environment file
generate_env_file() {
    log "Generating environment configuration..."
    
    cat > "$ENV_FILE" << EOF
# MetaPilot Program IDs - $NETWORK Network
# Generated: $(date)
# Deployment Version: $SCRIPT_VERSION

# Network Configuration
NEXT_PUBLIC_SOLANA_NETWORK=$NETWORK
NEXT_PUBLIC_SOLANA_RPC_URL=$RPC_URL

EOF

    # Add program IDs
    local programs=("dao-voting" "token-automation" "yield-optimizer" "transaction-scheduler")
    for program in "${programs[@]}"; do
        local program_id=$(jq -r ".programs[\"$program\"].program_id // \"\"" "$STATE_FILE" 2>/dev/null)
        local status=$(jq -r ".programs[\"$program\"].status // \"\"" "$STATE_FILE" 2>/dev/null)
        local env_var="NEXT_PUBLIC_$(echo "$program" | tr '[:lower:]' '[:upper:]' | tr '-' '_')_PROGRAM_ID"
        
        if [ "$status" = "deployed" ] && [ -n "$program_id" ]; then
            echo "$env_var=$program_id" >> "$ENV_FILE"
        else
            echo "# $env_var=pending # Status: $status" >> "$ENV_FILE"
        fi
    done
    
    # Add feature flags
    cat >> "$ENV_FILE" << EOF

# Feature Flags
NEXT_PUBLIC_AI_ENGINE_ENABLED=true
NEXT_PUBLIC_ENABLE_DAO_VOTING=true
NEXT_PUBLIC_ENABLE_TOKEN_AUTOMATION=true
NEXT_PUBLIC_ENABLE_YIELD_OPTIMIZATION=true
NEXT_PUBLIC_ENABLE_TRANSACTION_SCHEDULING=true
NEXT_PUBLIC_DEBUG_MODE=$([[ "$NETWORK" == "localnet" ]] && echo "true" || echo "false")
EOF
}

# Generate deployment summary
generate_summary() {
    log "Generating deployment summary..."
    
    local successful=$(jq -r '.metadata.successful_deployments' "$STATE_FILE")
    local failed=$(jq -r '.metadata.failed_deployments' "$STATE_FILE")
    local total=$(jq -r '.metadata.total_programs' "$STATE_FILE")
    
    cat > "$SUMMARY_FILE" << EOF
# MetaPilot Deployment Summary

## Network Information
- **Network**: $NETWORK
- **RPC URL**: $RPC_URL
- **Deployment Date**: $(date)
- **Deployer**: $WALLET_ADDRESS
- **Script Version**: $SCRIPT_VERSION

## Deployment Results
- **Total Programs**: $total
- **Successful**: $successful
- **Failed**: $failed
- **Success Rate**: $(echo "scale=1; $successful * 100 / $total" | bc -l)%

## Program Details

EOF

    # Add program details
    local programs=("dao-voting" "token-automation" "yield-optimizer" "transaction-scheduler")
    for program in "${programs[@]}"; do
        local program_id=$(jq -r ".programs[\"$program\"].program_id // \"N/A\"" "$STATE_FILE")
        local status=$(jq -r ".programs[\"$program\"].status // \"not_deployed\"" "$STATE_FILE")
        local signature=$(jq -r ".programs[\"$program\"].signature // \"N/A\"" "$STATE_FILE")
        local error=$(jq -r ".programs[\"$program\"].error // \"\"" "$STATE_FILE")
        
        local status_icon="❌"
        [ "$status" = "deployed" ] && status_icon="✅"
        [ "$status" = "unverified" ] && status_icon="⚠️"
        
        cat >> "$SUMMARY_FILE" << EOF
### $status_icon $(echo "$program" | tr '[:lower:]' '[:upper:]' | tr '-' ' ')
- **Program ID**: \`$program_id\`
- **Status**: $status
- **Signature**: \`$signature\`
EOF
        if [ -n "$error" ]; then
            echo "- **Error**: $error" >> "$SUMMARY_FILE"
        fi
        echo "" >> "$SUMMARY_FILE"
    done
    
    cat >> "$SUMMARY_FILE" << EOF
## Integration Files
- **State File**: \`$STATE_FILE\`
- **Environment File**: \`$ENV_FILE\`
- **Frontend Config**: Copy \`$ENV_FILE\` to \`../meta-pilot-frontend/.env.local\`

## Verification Commands
\`\`\`bash
# Verify program deployments
EOF

    for program in "${programs[@]}"; do
        local program_id=$(jq -r ".programs[\"$program\"].program_id // \"\"" "$STATE_FILE")
        if [ "$program_id" != "null" ] && [ -n "$program_id" ]; then
            echo "solana account $program_id  # $program" >> "$SUMMARY_FILE"
        fi
    done
    
    echo '```' >> "$SUMMARY_FILE"
}

# Copy to frontend
copy_to_frontend() {
    local frontend_paths=(
        "../meta-pilot-frontend/.env.local"
        "../../meta-pilot-frontend/.env.local"
        "../../../meta-pilot-frontend/.env.local"
    )
    
    for frontend_path in "${frontend_paths[@]}"; do
        if [ -d "$(dirname "$frontend_path")" ]; then
            cp "$ENV_FILE" "$frontend_path"
            success "Frontend configuration updated: $frontend_path"
            return 0
        fi
    done
    
    warning "Frontend directory not found. Please manually copy $ENV_FILE to your frontend .env.local"
}

# Main deployment process
main() {
    check_prerequisites
    configure_network
    setup_keypair
    
    # Build programs
    log "Building all programs..."
    if cargo build-sbf; then
        success "All programs built successfully"
    else
        error "Build failed"
    fi
    
    # Deploy programs
    local programs=("dao-voting" "token-automation" "yield-optimizer" "transaction-scheduler")
    local deployed_count=0
    local failed_count=0
    
    info "Deploying ${#programs[@]} programs to $NETWORK..."
    echo ""
    
    for program in "${programs[@]}"; do
        if deploy_program "$program"; then
            ((deployed_count++))
        else
            ((failed_count++))
        fi
        echo ""
    done
    
    # Generate files
    generate_env_file
    generate_summary
    copy_to_frontend
    
    # Final summary
    echo ""
    echo "🎉 Deployment Complete!"
    echo "======================="
    info "Network: $NETWORK"
    info "Programs deployed: $deployed_count/${#programs[@]}"
    info "Success rate: $(echo "scale=1; $deployed_count * 100 / ${#programs[@]}" | bc -l)%"
    echo ""
    info "Generated Files:"
    echo "  📄 $STATE_FILE"
    echo "  🔧 $ENV_FILE" 
    echo "  📋 $SUMMARY_FILE"
    echo ""
    
    if [ $failed_count -gt 0 ]; then
        warning "Some deployments failed. Check $SUMMARY_FILE for details."
        warning "Run with 'true' as 3rd parameter to force redeploy all programs."
    else
        success "All programs deployed successfully! 🚀"
    fi
    
    # Display program IDs
    echo "📝 Program IDs:"
    for program in "${programs[@]}"; do
        local program_id=$(jq -r ".programs[\"$program\"].program_id // \"Failed\"" "$STATE_FILE")
        printf "  %-25s %s\n" "$program:" "$program_id"
    done
    echo ""
    
    info "Next steps:"
    echo "  1. Test the deployed programs"
    echo "  2. Update your frontend with the new environment variables"
    echo "  3. Share the program IDs with your team"
    
    # Return appropriate exit code
    [ $failed_count -eq 0 ] && exit 0 || exit 1
}

# Run main function
main "$@"