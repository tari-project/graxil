// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/help/mining.rs
// Version: 1.0.1
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file provides comprehensive mining help, pool configuration guidance,
// wallet setup instructions, and troubleshooting tips for the SHA3x miner.
// It includes practical examples for different mining scenarios.
//
// Tree Location:
// - src/help/mining.rs (mining help and configuration guidance)
// - Depends on: none

/// Print comprehensive mining help
pub fn print_mining_help() {
    println!("MINING MODE:");
    println!("============");
    println!();
    println!("Mining mode connects to a Tari pool and contributes hashpower to find blocks.");
    println!("Successful shares earn Tari rewards based on your contribution.");
    println!();
    
    print_mining_requirements();
    println!();
    print_mining_examples();
    println!();
    print_pool_configuration();
    println!();
    print_wallet_setup();
}

/// Print mining requirements and setup
pub fn print_mining_requirements() {
    println!("MINING REQUIREMENTS:");
    println!("  - Valid Tari wallet address (for receiving rewards)");
    println!("  - Active internet connection");
    println!("  - Access to a Tari mining pool");
    println!("  - Sufficient system resources (CPU cooling)");
    println!();
    
    println!("BASIC MINING COMMAND:");
    println!("  sha3x-miner -u <WALLET_ADDRESS> -o <POOL:PORT> --threads <COUNT>");
    println!();
    
    println!("REQUIRED ARGUMENTS:");
    println!("  -u, --wallet     Your Tari wallet address for rewards");
    println!("  -o, --pool       Mining pool address (host:port format)");
    println!();
    
    println!("OPTIONAL ARGUMENTS:");
    println!("  -p, --password   Pool password (default: x)");
    println!("  -t, --threads    Number of threads to use");
    println!("  --worker         Worker identifier");
}

/// Get mining examples for different scenarios
pub fn get_mining_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Basic Pool Mining",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o tari-pool.com:4200 --threads 6"
        ),
        (
            "Mining with Custom Password",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o pool.tari.com:4200 -p worker-01 --threads 8"
        ),
        (
            "Mining with Specific Thread Count",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o pool.tari.com:4200 --threads 64"
        ),
        (
            "High-Performance Mining (Dual Xeon)",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o pool.tari.com:4200 --threads 72 --worker xeon-dual"
        ),
        (
            "Mining with Local Pool",
            "sha3x-miner -u 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW -o 192.168.1.100:4200 --threads 12"
        ),
    ]
}

/// Print mining examples with descriptions
pub fn print_mining_examples() {
    println!("MINING EXAMPLES:");
    println!();
    
    for (description, command) in get_mining_examples() {
        println!("{}:", description);
        println!("  {}", command);
        println!();
    }
}

/// Print pool configuration guidance
pub fn print_pool_configuration() {
    println!("POOL CONFIGURATION:");
    println!("===================");
    println!();
    
    println!("POOL ADDRESS FORMAT:");
    println!("  <hostname>:<port>     Example: tari-pool.com:4200");
    println!("  <ip>:<port>           Example: 192.168.1.100:4200");
    println!();
    
    println!("POPULAR TARI POOLS:");
    println!("  tari-pool.com:4200       Community pool (stable)");
    println!("  pool.tari.com:4200       Official Tari pool");
    println!("  localhost:4200           Local pool setup");
    println!();
    
    println!("POOL SELECTION CRITERIA:");
    println!("  - Low latency (ping time to pool)");
    println!("  - Stable uptime and reputation");
    println!("  - Fair fee structure (typically 1-2%)");
    println!("  - Good payout frequency");
    println!("  - Active community support");
    println!();
    
    println!("WORKER NAMING BEST PRACTICES:");
    println!("  Good:  rig-01, office-pc, server-main, xeon-dual");
    println!("  Good:  Use descriptive, unique names per machine");
    println!("  Avoid: Spaces, special characters, generic names");
    println!();
    
    print_pool_troubleshooting();
}

/// Print pool-specific troubleshooting
pub fn print_pool_troubleshooting() {
    println!("POOL CONNECTION TROUBLESHOOTING:");
    println!("=================================");
    println!();
    
    println!("CONNECTION ISSUES:");
    println!("  - Check pool address and port number");
    println!("  - Verify pool is online (check pool website/discord)");
    println!("  - Test with ping: ping tari-pool.com");
    println!("  - Check firewall/router settings");
    println!("  - Try different pool if current is down");
    println!();
    
    println!("AUTHENTICATION ISSUES:");
    println!("  - Verify wallet address is correct");
    println!("  - Ensure wallet address is for Tari mainnet");
    println!("  - Check worker name doesn't contain invalid characters");
    println!("  - Contact pool operator if login keeps failing");
    println!();
    
    println!("PERFORMANCE ISSUES:");
    println!("  - High ping times (>100ms): Try closer pool");
    println!("  - Share rejections: Check system clock sync");
    println!("  - Low hashrate: Run benchmark to test hardware");
    println!("  - Disconnections: Check network stability");
}

/// Print wallet setup guidance
pub fn print_wallet_setup() {
    println!("WALLET SETUP:");
    println!("=============");
    println!();
    
    println!("WALLET REQUIREMENTS:");
    println!("  - Valid Tari wallet address (starts with '12' or '14')");
    println!("  - '12' prefix = One-sided address (most common)");
    println!("  - '14' prefix = Interactive address");
    println!("  - Wallet must be synchronized with network");
    println!("  - Address should be for Tari mainnet (not testnet)");
    println!();
    
    println!("GETTING A TARI WALLET:");
    println!("  Mobile: Tari Aurora Wallet (iOS/Android)");
    println!("  Desktop: Tari Console Wallet (command-line)");
    println!("  Web: Tari Universe (web-based wallet)");
    println!();
    
    println!("WALLET ADDRESS EXAMPLE:");
    println!("  125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW");
    println!("  ^");
    println!("  One-sided address starting with '12' (Base58 encoded, ~87 characters)");
    println!();
    
    println!("WALLET SECURITY:");
    println!("  - Keep your seed phrase/private keys secure");
    println!("  - Never share your private keys with anyone");
    println!("  - Use only your own wallet address for mining");
    println!("  - Verify address is correct before starting mining");
    println!();
    
    print_reward_expectations();
}

/// Print reward and earnings guidance
pub fn print_reward_expectations() {
    println!("MINING REWARDS & EXPECTATIONS:");
    println!("===============================");
    println!();
    
    println!("REWARD STRUCTURE:");
    println!("  - Rewards based on valid shares submitted");
    println!("  - Pool fees typically 1-2% of earnings");
    println!("  - Payouts depend on pool policy (daily/weekly)");
    println!("  - Higher hashrate = more shares = more rewards");
    println!();
    
    println!("FACTORS AFFECTING EARNINGS:");
    println!("  - Your hashrate contribution");
    println!("  - Network difficulty");
    println!("  - Pool luck and efficiency");
    println!("  - Tari block reward amount");
    println!("  - Pool fee structure");
    println!();
    
    println!("REALISTIC EXPECTATIONS:");
    println!("  - Mining is competitive - profits depend on many factors");
    println!("  - Consider electricity costs vs rewards");
    println!("  - Pool mining provides steady income vs solo mining");
    println!("  - Start small and scale based on profitability");
    println!();
    
    println!("MONITORING YOUR MINING:");
    println!("  - Track hashrate consistency");
    println!("  - Monitor share acceptance rate (should be >95%)");
    println!("  - Check pool dashboard for detailed stats");
    println!("  - Watch for hardware issues (temperature, stability)");
}

/// Print mining optimization tips
pub fn print_mining_optimization() {
    println!("MINING OPTIMIZATION:");
    println!("====================");
    println!();
    
    println!("HARDWARE OPTIMIZATION:");
    println!("  - Use benchmark mode to find optimal thread count");
    println!("  - Ensure adequate cooling for sustained operation");
    println!("  - Monitor CPU temperatures (keep under 85C)");
    println!("  - Use high-performance power settings");
    println!("  - Close unnecessary background applications");
    println!();
    
    println!("NETWORK OPTIMIZATION:");
    println!("  - Choose pool with lowest latency");
    println!("  - Use wired internet connection if possible");
    println!("  - Ensure stable internet connection");
    println!("  - Monitor for connection drops");
    println!();
    
    println!("SYSTEM MAINTENANCE:");
    println!("  - Regular system updates and reboots");
    println!("  - Monitor system logs for errors");
    println!("  - Keep mining software updated");
    println!("  - Backup wallet and important data");
    println!();
    
    println!("PROFITABILITY TIPS:");
    println!("  - Calculate electricity costs vs rewards");
    println!("  - Consider mining during off-peak electricity hours");
    println!("  - Monitor Tari price and network difficulty");
    println!("  - Join mining communities for tips and updates");
}

// Changelog:
// - v1.0.1 (2025-06-15): Fixed Unicode character compilation errors.
//   - Replaced all Unicode box drawing characters with ASCII equivalents
//   - Removed all emoji characters causing compilation issues
//   - Simplified formatting to use standard dashes and ASCII characters
//   - Updated Tari address format to use correct prefixes (12/14)
// - v1.0.0 (2025-06-15): Initial mining help implementation.
//   - Purpose: Provides comprehensive mining guidance with pool configuration,
//     wallet setup, troubleshooting, and optimization strategies.
//   - Features: Mining requirements, practical examples, pool selection criteria,
//     wallet setup guidance, reward expectations, and optimization tips for
//     successful Tari mining operations.
//   - Note: This module serves as a complete mining reference, helping users
//     set up and optimize their mining operations effectively.