#!/usr/bin/env node

/**
 * MetaPilot Professional Solana Program Deployment Script
 * Optimized for cost efficiency with reduced storage requirements
 */

const { Connection, Keypair, PublicKey, SystemProgram, Transaction } = require('@solana/web3.js');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync, spawn } = require('child_process');

// Configuration
const DEPLOYMENT_CONFIG = {
    network: process.env.SOLANA_NETWORK || process.argv[2] || 'devnet',
    rpcUrl: process.env.RPC_URL,
    payerKeypairPath: process.env.PAYER_KEYPAIR_PATH || 'keys/deployer.json',
    deploymentOutputPath: './deployments'
};

// Program configurations with optimized storage
const PROGRAMS = [
    {
        name: 'dao-voting',
        binaryPath: 'target/deploy/dao_voting.so',
        keypairPath: 'target/deploy/dao_voting-keypair.json',
        storageSize: 4096, // Minimal 4KB storage
        description: 'MetaPilot DAO Voting Automation Program'
    },
    {
        name: 'token-automation',
        binaryPath: 'target/deploy/token_automation.so',
        keypairPath: 'target/deploy/token_automation-keypair.json',
        storageSize: 4096, // Minimal 4KB storage
        description: 'MetaPilot Token Trading and Swap Automation Program'
    },
    {
        name: 'yield-optimizer',
        binaryPath: 'target/deploy/yield_optimizer.so',
        keypairPath: 'target/deploy/yield_optimizer-keypair.json',
        storageSize: 4096, // Minimal 4KB storage
        description: 'MetaPilot Yield Farming and DeFi Strategy Optimization Program'
    },
    {
        name: 'transaction-scheduler',
        binaryPath: 'target/deploy/transaction_scheduler.so',
        keypairPath: 'target/deploy/transaction_scheduler-keypair.json',
        storageSize: 4096, // Minimal 4KB storage
        description: 'MetaPilot Transaction Scheduling and Gas Optimization Program'
    }
];

class MetaPilotDeployer {
    constructor() {
        this.connection = null;
        this.payer = null;
        this.deploymentResults = [];
        this.existingDeployments = {};
        this.totalCost = 0;
        
        console.log('🚀 MetaPilot Professional Deployment System v2.0');
        console.log('==================================================');
        
        this.setupConnection();
        this.setupPayer();
        this.loadExistingDeployments();
    }

    setupConnection() {
        const rpcUrl = DEPLOYMENT_CONFIG.rpcUrl || this.getDefaultRpcUrl();
        console.log(`🌐 Network: ${DEPLOYMENT_CONFIG.network}`);
        console.log(`📡 RPC: ${rpcUrl}`);
        
        this.connection = new Connection(rpcUrl, 'confirmed');
    }

    getDefaultRpcUrl() {
        switch (DEPLOYMENT_CONFIG.network) {
            case 'devnet':
                return 'https://api.devnet.solana.com';
            case 'testnet':
                return 'https://api.testnet.solana.com';
            case 'mainnet-beta':
                return 'https://api.mainnet-beta.solana.com';
            case 'localnet':
            case 'localhost':
                return 'http://127.0.0.1:8899';
            default:
                throw new Error(`Unknown network: ${DEPLOYMENT_CONFIG.network}`);
        }
    }

    setupPayer() {
        const keypairPath = DEPLOYMENT_CONFIG.payerKeypairPath;
        
        console.log(`🔑 Loading keypair: ${keypairPath}`);
        
        try {
            if (!fs.existsSync(keypairPath)) {
                throw new Error(`Keypair file not found: ${keypairPath}`);
            }
            
            const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
            this.payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));
            console.log(`💰 Deployer: ${this.payer.publicKey.toString()}`);
        } catch (error) {
            console.error('❌ Failed to load payer keypair:', error.message);
            throw error;
        }
    }

    loadExistingDeployments() {
        try {
            const networkPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
            const statePath = path.join(networkPath, 'deployment-state.json');
            
            if (fs.existsSync(statePath)) {
                const data = JSON.parse(fs.readFileSync(statePath, 'utf8'));
                
                if (data.programs) {
                    Object.keys(data.programs).forEach(name => {
                        const program = data.programs[name];
                        if (program.status === 'deployed') {
                            this.existingDeployments[name] = program;
                        }
                    });
                }
                console.log(`📋 Found ${Object.keys(this.existingDeployments).length} existing deployments`);
            }
        } catch (error) {
            console.log('📝 Starting fresh deployment');
            this.existingDeployments = {};
        }
    }

    async checkBalance() {
        const balance = await this.connection.getBalance(this.payer.publicKey);
        const solBalance = balance / 1e9;
        
        console.log(`\n💸 Current Balance: ${solBalance.toFixed(4)} SOL`);
        
        // Estimate costs for each program and select deployable ones
        this.deployablePrograms = [];
        let runningCost = 0;
        
        for (const program of PROGRAMS) {
            if (!this.existingDeployments[program.name]) {
                // Estimate cost for this program
                const rentExemption = await this.connection.getMinimumBalanceForRentExemption(program.storageSize);
                const programCost = (rentExemption / 1e9) + 0.6; // 0.6 SOL for deployment fees
                
                console.log(`📊 ${program.name}: ~${programCost.toFixed(4)} SOL`);
                
                if (runningCost + programCost <= solBalance * 0.95) { // Leave 5% buffer
                    this.deployablePrograms.push(program);
                    runningCost += programCost;
                    console.log(`   ✅ Can deploy (running total: ${runningCost.toFixed(4)} SOL)`);
                } else {
                    console.log(`   ⏸️  Insufficient funds (would need ${(runningCost + programCost).toFixed(4)} SOL)`);
                }
            } else {
                this.deployablePrograms.push(program); // Already deployed
            }
        }
        
        console.log(`\n📋 Will deploy ${this.deployablePrograms.length}/${PROGRAMS.length} programs`);
        console.log(`💰 Estimated cost: ${runningCost.toFixed(4)} SOL`);
        
        if (this.deployablePrograms.length === 0) {
            throw new Error('No programs can be deployed with current balance');
        }
        
        return true;
    }

    validateBuildFiles() {
        console.log('\n🔍 Validating build files...');
        
        const missingFiles = [];
        PROGRAMS.forEach(program => {
            if (!fs.existsSync(program.binaryPath)) {
                missingFiles.push(program.binaryPath);
            }
            if (!fs.existsSync(program.keypairPath)) {
                missingFiles.push(program.keypairPath);
            }
        });

        if (missingFiles.length > 0) {
            console.error('❌ Missing files:');
            missingFiles.forEach(file => console.error(`   - ${file}`));
            throw new Error('Missing build artifacts');
        }

        // Check binary sizes
        PROGRAMS.forEach(program => {
            const stats = fs.statSync(program.binaryPath);
            const sizeKB = Math.round(stats.size / 1024);
            console.log(`📦 ${program.name}: ${sizeKB} KB`);
            
            // Warn if binary is very large (>500KB)
            if (stats.size > 500 * 1024) {
                console.warn(`⚠️  Large binary detected: ${program.name} (${sizeKB} KB)`);
                console.log('💡 Consider optimizing with: cargo build-sbf --release');
            }
        });

        console.log('✅ All build files validated');
    }

    async deployProgram(programInfo) {
        console.log(`\n🚀 Deploying ${programInfo.name}...`);
        
        // Check if already deployed
        if (this.existingDeployments[programInfo.name]) {
            const existing = this.existingDeployments[programInfo.name];
            console.log(`✅ Already deployed: ${existing.program_id}`);
            this.deploymentResults.push({
                ...existing,
                skipped: true
            });
            return existing;
        }

        try {
            // Load program keypair
            const programKeypairData = JSON.parse(fs.readFileSync(programInfo.keypairPath, 'utf8'));
            const programKeypair = Keypair.fromSecretKey(Uint8Array.from(programKeypairData));
            
            console.log(`📝 Program ID: ${programKeypair.publicKey.toString()}`);
            
            // Check if program account exists
            const programAccount = await this.connection.getAccountInfo(programKeypair.publicKey);
            
            if (programAccount && programAccount.executable) {
                console.log('⏭️  Program already exists on-chain, upgrading...');
                
                // Use solana program deploy command for upgrade
                const deployResult = await this.deployWithCLI(programInfo, programKeypair.publicKey.toString());
                return deployResult;
            } else {
                console.log('🆕 New program deployment');
                
                // Deploy new program
                const deployResult = await this.deployWithCLI(programInfo);
                return deployResult;
            }

        } catch (error) {
            console.error(`❌ Failed to deploy ${programInfo.name}:`, error.message);
            
            // Save failed deployment info
            const failedResult = {
                name: programInfo.name,
                status: 'failed',
                error: error.message,
                timestamp: new Date().toISOString()
            };
            
            this.deploymentResults.push(failedResult);
            this.saveFailedDeployment(failedResult);
            
            throw error;
        }
    }

    async deployWithCLI(programInfo, existingProgramId = null) {
        return new Promise((resolve, reject) => {
            const args = ['program', 'deploy', programInfo.binaryPath];
            
            // Use existing program ID if upgrading
            if (existingProgramId) {
                args.push('--program-id', existingProgramId);
            } else if (fs.existsSync(programInfo.keypairPath)) {
                args.push('--program-id', programInfo.keypairPath);
            }
            
            // Add buffer size optimization - use actual binary size + small buffer
            const binaryStats = require('fs').statSync(programInfo.binaryPath);
            const optimalSize = Math.max(binaryStats.size * 1.1, 600000); // 10% buffer or 600KB min
            args.push('--max-len', Math.floor(optimalSize).toString());
            args.push('--output', 'json');

            console.log(`🔨 Command: solana ${args.join(' ')}`);

            const solana = spawn('solana', args, {
                stdio: ['pipe', 'pipe', 'pipe']
            });

            let stdout = '';
            let stderr = '';

            solana.stdout.on('data', (data) => {
                stdout += data.toString();
            });

            solana.stderr.on('data', (data) => {
                stderr += data.toString();
                // Show progress for long deployments
                if (data.toString().includes('Writing')) {
                    process.stdout.write('.');
                }
            });

            solana.on('close', (code) => {
                if (code === 0) {
                    try {
                        const result = JSON.parse(stdout.trim());
                        const deploymentResult = {
                            name: programInfo.name,
                            program_id: result.programId,
                            status: 'deployed',
                            signature: result.signature,
                            timestamp: new Date().toISOString(),
                            description: programInfo.description,
                            binary_path: programInfo.binaryPath,
                            storage_size: programInfo.storageSize,
                            deployer: this.payer.publicKey.toString(),
                            network: DEPLOYMENT_CONFIG.network
                        };

                        console.log(`\n✅ ${programInfo.name} deployed successfully!`);
                        console.log(`📍 Program ID: ${result.programId}`);
                        console.log(`🔗 Signature: ${result.signature}`);

                        this.deploymentResults.push(deploymentResult);
                        this.saveSuccessfulDeployment(deploymentResult);
                        
                        resolve(deploymentResult);
                    } catch (parseError) {
                        console.log('\n❌ Failed to parse deployment result');
                        reject(new Error('Failed to parse deployment result'));
                    }
                } else {
                    console.log(`\n❌ Deployment failed with code ${code}`);
                    console.error('Error output:', stderr);
                    reject(new Error(`Deployment failed: ${stderr}`));
                }
            });
        });
    }

    async deployAllPrograms() {
        console.log('\n🎯 Starting MetaPilot deployment...');
        
        // Prerequisites
        this.validateBuildFiles();
        await this.checkBalance();

        let successCount = 0;
        let failureCount = 0;

        // Deploy each deployable program
        for (const programInfo of this.deployablePrograms) {
            try {
                await this.deployProgram(programInfo);
                successCount++;
                
                // Brief pause between deployments
                if (PROGRAMS.indexOf(programInfo) < PROGRAMS.length - 1) {
                    console.log('⏳ Cooling down...');
                    await new Promise(resolve => setTimeout(resolve, 3000));
                }
            } catch (error) {
                failureCount++;
                console.error(`💥 ${programInfo.name} deployment failed`);
                
                // Continue with other deployments
                continue;
            }
        }

        const remainingPrograms = PROGRAMS.length - this.deployablePrograms.length;
        console.log(`\n📊 Deployment Summary: ${successCount} succeeded, ${failureCount} failed`);
        
        if (remainingPrograms > 0) {
            console.log(`⏸️  ${remainingPrograms} programs deferred due to insufficient balance`);
            console.log('💡 Get more SOL and run deployment again to complete remaining programs');
        }
    }

    saveSuccessfulDeployment(result) {
        // Save individual deployment
        const networkPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
        if (!fs.existsSync(networkPath)) {
            fs.mkdirSync(networkPath, { recursive: true });
        }

        // Update environment file immediately
        this.updateEnvironmentFile(result);
        
        // Update deployment state
        this.updateDeploymentState();
        
        console.log(`💾 ${result.name} deployment saved`);
    }

    saveFailedDeployment(result) {
        const networkPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
        if (!fs.existsSync(networkPath)) {
            fs.mkdirSync(networkPath, { recursive: true });
        }

        // Update deployment state with failure
        this.updateDeploymentState();
    }

    updateDeploymentState() {
        const networkPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
        const statePath = path.join(networkPath, 'deployment-state.json');

        const programs = {};
        this.deploymentResults.forEach(result => {
            programs[result.name] = result;
        });

        const successful = this.deploymentResults.filter(r => r.status === 'deployed').length;
        const failed = this.deploymentResults.filter(r => r.status === 'failed').length;

        const stateData = {
            network: DEPLOYMENT_CONFIG.network,
            deployment_version: "2.0.0",
            programs: programs,
            metadata: {
                created_at: new Date().toISOString(),
                last_updated: new Date().toISOString(),
                deployer_keypair: DEPLOYMENT_CONFIG.payerKeypairPath,
                deployer: this.payer.publicKey.toString(),
                total_programs: PROGRAMS.length,
                successful_deployments: successful,
                failed_deployments: failed,
                success_rate: `${Math.round((successful / PROGRAMS.length) * 100)}%`,
                deployment_tool: "MetaPilot Professional Deployment v2.0",
                rpc_url: this.getDefaultRpcUrl()
            }
        };

        fs.writeFileSync(statePath, JSON.stringify(stateData, null, 2));
    }

    updateEnvironmentFile(result) {
        const networkPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
        const envPath = path.join(networkPath, 'program-ids.env');
        
        let envContent = '';
        
        // Read existing content
        if (fs.existsSync(envPath)) {
            envContent = fs.readFileSync(envPath, 'utf8');
        } else {
            // Create header for new file
            envContent = `# MetaPilot Program IDs - ${DEPLOYMENT_CONFIG.network} Network
# Generated: ${new Date().toISOString()}
# Deployment Version: 2.0.0

# Network Configuration
NEXT_PUBLIC_SOLANA_NETWORK=${DEPLOYMENT_CONFIG.network}
NEXT_PUBLIC_SOLANA_RPC_URL=${this.getDefaultRpcUrl()}

# Program IDs
`;
        }

        // Add or update program ID
        const envKey = `NEXT_PUBLIC_${result.name.toUpperCase().replace('-', '_')}_PROGRAM_ID`;
        
        // Remove existing entry
        envContent = envContent.split('\n')
            .filter(line => !line.startsWith(`${envKey}=`))
            .join('\n');

        // Add new entry
        envContent += `${envKey}=${result.program_id}\n`;

        fs.writeFileSync(envPath, envContent);
    }

    async copyToFrontend() {
        const envPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network, 'program-ids.env');
        
        // Try multiple possible frontend paths
        const frontendPaths = [
            '../meta-pilot-frontend/.env.local',
            '../../meta-pilot-frontend/.env.local',
            '../../../meta-pilot-frontend/.env.local'
        ];
        
        for (const frontendPath of frontendPaths) {
            if (fs.existsSync(path.dirname(frontendPath))) {
                fs.copyFileSync(envPath, frontendPath);
                console.log(`📄 Frontend config updated: ${frontendPath}`);
                return;
            }
        }
        
        console.log(`⚠️  Frontend directory not found. Manual copy required:`);
        console.log(`   cp ${envPath} your-frontend/.env.local`);
    }

    printFinalSummary() {
        const successful = this.deploymentResults.filter(r => r.status === 'deployed');
        const failed = this.deploymentResults.filter(r => r.status === 'failed');
        const skipped = this.deploymentResults.filter(r => r.skipped);

        console.log('\n🎉 MetaPilot Deployment Complete!');
        console.log('=====================================');
        console.log(`📊 Network: ${DEPLOYMENT_CONFIG.network}`);
        console.log(`💰 Deployer: ${this.payer.publicKey.toString()}`);
        console.log(`✅ Successful: ${successful.length}`);
        console.log(`⏭️  Skipped: ${skipped.length}`);
        console.log(`❌ Failed: ${failed.length}`);
        console.log(`📈 Success Rate: ${Math.round((successful.length / PROGRAMS.length) * 100)}%`);

        if (successful.length > 0) {
            console.log('\n📝 Successfully Deployed Programs:');
            successful.forEach((result, index) => {
                console.log(`${index + 1}. ${result.name.toUpperCase()}`);
                console.log(`   📍 ${result.program_id}`);
                console.log(`   🔗 ${result.signature}`);
            });
        }

        if (failed.length > 0) {
            console.log('\n❌ Failed Deployments:');
            failed.forEach((result, index) => {
                console.log(`${index + 1}. ${result.name}: ${result.error}`);
            });
        }

        const deploymentPath = path.join(DEPLOYMENT_CONFIG.deploymentOutputPath, DEPLOYMENT_CONFIG.network);
        console.log(`\n📁 Deployment files: ${deploymentPath}`);
        console.log('🔗 State: deployment-state.json');
        console.log('🔧 Environment: program-ids.env');
    }
}

// Main execution
async function main() {
    try {
        const deployer = new MetaPilotDeployer();
        await deployer.deployAllPrograms();
        await deployer.copyToFrontend();
        deployer.printFinalSummary();
        
        console.log('\n🚀 Deployment completed successfully!');
        process.exit(0);
        
    } catch (error) {
        console.error('\n💥 Deployment failed:', error.message);
        process.exit(1);
    }
}

// Help message
if (process.argv.includes('--help') || process.argv.includes('-h')) {
    console.log(`
🚀 MetaPilot Professional Deployment Script

Usage:
  node professional-deploy.js [network]
  
Networks:
  localnet     Local Solana validator
  devnet       Solana devnet (default)
  testnet      Solana testnet  
  mainnet-beta Solana mainnet

Environment Variables:
  SOLANA_NETWORK          Target network
  RPC_URL                 Custom RPC endpoint
  PAYER_KEYPAIR_PATH      Deployer keypair path

Examples:
  node professional-deploy.js devnet
  PAYER_KEYPAIR_PATH=./my-key.json node professional-deploy.js testnet
    `);
    process.exit(0);
}

// Run deployment
if (require.main === module) {
    main().catch(console.error);
}

module.exports = { MetaPilotDeployer, DEPLOYMENT_CONFIG };