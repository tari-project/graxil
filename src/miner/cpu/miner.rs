// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/cpu/miner.rs
// Version: 2.0.4-dns
// Developer: OIEIEIO <oieieio@protonmail.com>

use crate::core::{parse_target_difficulty, Algorithm, PoolJob, MiningJob};
use crate::miner::stats::MinerStats;
use crate::pool::{PoolClient, protocol::StratumProtocol};
use crate::Result;
use num_cpus;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::mpsc;
use tokio::sync::broadcast::{self, Sender as BroadcastSender};
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use std::collections::HashMap;

// Explicit fully qualified import to bypass resolution issues
use super::thread::start_mining_thread;

pub struct CpuMiner {
    wallet_address: String,
    pool_address: String,
    worker_name: String,
    num_threads: usize,
    stats: Arc<MinerStats>,
    pool_client: PoolClient,
    algo: Algorithm,
    current_difficulty: Arc<AtomicU64>,
    current_jobs: Arc<Mutex<HashMap<String, PoolJob>>>,
    last_job_time: Arc<Mutex<Instant>>,
}

impl CpuMiner {
    pub fn new(
        wallet_address: String,
        pool_address: String,
        worker_name: String,
        num_threads: usize,
        algo: Algorithm,
    ) -> Self {
        let actual_threads = if num_threads == 0 {
            num_cpus::get()
        } else {
            num_threads
        };

        let mut stats = MinerStats::new(actual_threads);
        stats.set_algorithm(algo);

        Self {
            wallet_address,
            pool_address,
            worker_name,
            num_threads: actual_threads,
            stats: Arc::new(stats),
            pool_client: PoolClient::new(),
            algo,
            current_difficulty: Arc::new(AtomicU64::new(1)),
            current_jobs: Arc::new(Mutex::new(HashMap::new())),
            last_job_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Get access to miner statistics for web dashboard
    /// Returns a reference to the internal MinerStats for real-time monitoring
    pub fn get_stats(&self) -> Arc<MinerStats> {
        Arc::clone(&self.stats)
    }

    /// Test SV2 Noise connection to JDS
    pub async fn test_sv2_connection(&self) -> Result<()> {
        info!("🔧 Testing basic TCP connection to JDS at {}...", self.pool_address);
        
        // Use the new string-based connection method
        let _stream = self.pool_client.connect_str(&self.pool_address).await
            .map_err(|e| format!("Failed to connect to {}: {}", self.pool_address, e))?;
        
        info!("✅ TCP connection to JDS successful!");
        info!("📝 Note: Full SV2 Noise handshake implementation in progress");
        info!("🚀 This confirms your JDS is running and accepting connections");
        
        Ok(())
    }

    // Keep SHA3x functionality unchanged
    async fn connect_to_pool(&self) -> Result<tokio::net::TcpStream> {
        Ok(self.pool_client.connect_str(&self.pool_address).await?)
    }

    async fn login(&self, writer: &mut tokio::net::tcp::OwnedWriteHalf) -> Result<()> {
        // Only SHA3x login now
        let login_msg = StratumProtocol::to_message(
            StratumProtocol::create_login_request(&self.wallet_address, &self.worker_name, self.algo)
        );
        writer.write_all(login_msg.as_bytes()).await?;
        writer.flush().await?;
        info!("📤 Sent SHA3x login request");
        Ok(())
    }

    async fn handle_pool_message(
        &self,
        message: &str,
        job_tx: &BroadcastSender<MiningJob>,
    ) -> Result<()> {
        debug!("📨 Raw pool message: {}", message);
        let response: Value = serde_json::from_str(message)?;

        if let Some(method) = response.get("method").and_then(|m| m.as_str()) {
            match method {
                "job" => {
                    debug!("Processing SHA3x job message: {:?}", response);
                    if let Some(params) = response.get("params").and_then(|p| p.as_object()) {
                        self.handle_new_job(params, job_tx).await?;
                        if let Some(diff) = params.get("difficulty").and_then(|d| d.as_u64()) {
                            self.stats.add_activity(format!("🔧 VarDiff update: {}", MinerStats::format_number(diff)));
                            info!("🔧 VarDiff job update received");
                        }
                    }
                }
                _ => {
                    debug!("Unknown method: {}", method);
                }
            }
        } else if let Some(result) = response.get("result") {
            debug!("Result response: {:?}", result);
            if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
                match id {
                    1 => {
                        info!("✅ SHA3x login successful");
                        self.stats.add_activity("🔐 Connected successfully".to_string());
                        if let Some(job_params) = result.get("job").and_then(|j| j.as_object()) {
                            debug!("Found job in login response: {:?}", job_params);
                            self.handle_new_job(job_params, job_tx).await?;
                        }
                    }
                    id if id >= 100 => {
                        let thread_id = (id - 100) as usize % self.num_threads;
                        debug!("Share response for ID {} (thread {}): {:?}", id, thread_id, result);
                        let accepted = if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
                            matches!(status.to_lowercase().as_str(), "ok" | "accepted")
                        } else if result.is_null() {
                            info!("✅ Share accepted (null response)");
                            true
                        } else if let Some(accepted) = result.as_bool() {
                            accepted
                        } else {
                            error!("❌ Unknown share response format: {:?}", result);
                            false
                        };

                        if accepted {
                            self.stats.shares_accepted.fetch_add(1, Ordering::Relaxed);
                            info!("✅ Share accepted by pool");
                            self.stats.add_activity(format!("✅ Share accepted from thread {}", thread_id));
                        } else {
                            self.stats.shares_rejected.fetch_add(1, Ordering::Relaxed);
                            info!("❌ Share rejected from thread {}", thread_id);
                            self.stats.add_activity(format!("❌ Share rejected from thread {}", thread_id));
                        }

                        if thread_id < self.stats.thread_stats.len() {
                            self.stats.thread_stats[thread_id].record_share(0, accepted);
                        }
                    }
                    _ => {}
                }
            }
        } else if let Some(error) = response.get("error") {
            error!("❌ Pool error: {:?}", error);
            self.stats.add_activity(format!("🚫 Pool error: {}", error));
        } else {
            debug!("Unknown pool message: {:?}", response);
        }

        Ok(())
    }

    async fn handle_new_job(
        &self,
        job_data: &serde_json::Map<String, Value>,
        job_tx: &BroadcastSender<MiningJob>,
    ) -> Result<()> {
        let job: PoolJob = serde_json::from_value(Value::Object(job_data.clone()))?;
        
        // Only handle SHA3x jobs now
        let header_template = hex::decode(&job.blob.unwrap_or_default())?;
        let target_difficulty = job.difficulty.unwrap_or_else(|| parse_target_difficulty(&job.target, self.algo));
        
        let mining_job = MiningJob {
            job_id: job.job_id.clone(),
            mining_hash: header_template,
            target_difficulty,
            height: job.height,
            algo: Algorithm::Sha3x,
            prev_hash: None,
            merkle_root: None,
            version: None,
            ntime: None,
            nbits: None,
            merkle_path: None,
            target: None,
        };

        // Update MinerStats with job data for web dashboard
        self.stats.update_job(job.job_id.clone(), job.height, target_difficulty);

        job_tx.send(mining_job)?;
        info!("📋 New job sent: {} (height: {}, difficulty: {})", 
            job.job_id, job.height, MinerStats::format_number(target_difficulty));
        self.stats.add_activity(format!(
            "📋 New job: {} (height: {}, difficulty: {})",
            &job.job_id[..8.min(job.job_id.len())], job.height, MinerStats::format_number(target_difficulty)
        ));

        *self.last_job_time.lock().await = Instant::now();
        Ok(())
    }

    fn start_share_submitter(
        miner: Arc<Self>,
        writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
        mut share_rx: mpsc::UnboundedReceiver<(String, String, String, usize, u64, String, u32)>,
    ) {
        let wallet_address = miner.wallet_address.clone();
        let algo = miner.algo;
        static SUBMIT_ID: AtomicU32 = AtomicU32::new(100);

        tokio::spawn(async move {
            while let Some((job_id, nonce, result, _thread_id, _difficulty, _extranonce2, _ntime)) = share_rx.recv().await {
                let submit_request = StratumProtocol::create_submit_request(
                    &wallet_address,
                    &job_id,
                    &nonce,
                    &result,
                    SUBMIT_ID.fetch_add(1, Ordering::SeqCst) as u64,
                    algo,
                    None, // No extranonce2 for SHA3x
                    None, // No ntime for SHA3x
                );
                let message = StratumProtocol::to_message(submit_request);
                if message.is_empty() {
                    error!("Failed to create submit message for job {}", job_id);
                    continue;
                }

                info!("📤 Submitting SHA3x share: job_id={}, nonce={}, result={}", 
                    job_id, nonce, result);

                let mut writer = writer.lock().await;
                if let Err(e) = writer.write_all(message.as_bytes()).await {
                    error!("Failed to submit share: {}", e);
                }
            }
        });
    }

    fn start_stats_printer(miner: Arc<Self>) {
        let stats = Arc::clone(&miner.stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                let dashboard_id = format!("{:016x}", rand::random::<u64>());
                stats.display_dashboard(&dashboard_id);
            }
        });
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        // SHA3x mining only now
        if self.algo != Algorithm::Sha3x {
            return Err("Only SHA3x algorithm supported in this version".into());
        }

        let stream = self.connect_to_pool().await?;
        info!("✅ Connected to SHA3x pool");
        self.stats.add_activity("🔐 Connected to pool".to_string());

        let (reader, writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(writer));

        self.login(&mut *writer.lock().await).await?;
        info!("🔐 SHA3x login request sent");

        let (job_tx, _) = broadcast::channel(16);
        let (share_tx, share_rx) = mpsc::unbounded_channel::<(String, String, String, usize, u64, String, u32)>();

        debug!("Starting {} mining threads", self.num_threads);
        for thread_id in 0..self.num_threads {
            let job_rx = job_tx.subscribe();
            let share_tx = share_tx.clone();
            let thread_stats = Arc::clone(&self.stats.thread_stats[thread_id]);
            let stats = Arc::clone(&self.stats);
            debug!("Spawning thread {}", thread_id); // Debug
            start_mining_thread(thread_id, self.num_threads, job_rx, share_tx, thread_stats, stats);
        }

        CpuMiner::start_share_submitter(self.clone(), Arc::clone(&writer), share_rx);
        CpuMiner::start_stats_printer(self.clone());

        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    self.handle_pool_message(&line, &job_tx).await?;
                }
                Ok(None) => {
                    info!("📡 Connection closed, attempting reconnect...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let new_stream = self.connect_to_pool().await?;
                    let (new_reader, new_writer) = new_stream.into_split();
                    *writer.lock().await = new_writer;
                    lines = BufReader::new(new_reader).lines();
                    self.login(&mut *writer.lock().await).await?;
                    info!("🔄 Reconnected to pool");
                }
                Err(e) => {
                    error!("📡 Error reading from pool: {}, attempting reconnect...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let new_stream = self.connect_to_pool().await?;
                    let (new_reader, new_writer) = new_stream.into_split();
                    *writer.lock().await = new_writer;
                    lines = BufReader::new(new_reader).lines();
                    self.login(&mut *writer.lock().await).await?;
                    info!("🔄 Reconnected to pool after error");
                }
            }
        }
    }
}

// Changelog:
// - v2.0.4-dns (2025-06-23): Added DNS resolution support.
//   - Changed pool_address field from SocketAddr to String
//   - Updated constructor to accept String pool address
//   - Modified connect_to_pool and test_sv2_connection to use connect_str
//   - Now supports both IP addresses and domain names like pool.sha3x.supportxtm.com:6118
//   - Maintains backward compatibility with IP:port format
// - v2.0.3-web (2025-06-23): Fixed start_mining_thread import.
//   - Changed import to use super::thread::start_mining_thread to bypass resolution issues.
//   - Added debug logs in run to trace thread spawning.
//   - Kept stats.update_job call in handle_new_job for web dashboard.
//   - Kept log message as "New job sent:".
//   - Compatible with miner_stats.rs v1.0.9, web_server.rs v1.0.1, thread.rs v1.1.4.
// - v2.0.2-web (2025-06-22): Added job data for web dashboard.
//   - Added stats.update_job call in handle_new_job to update MinerStats with job data.
//   - Changed log message from "SHA3x job sent to threads" to "New job sent:".
//   - Ensures job ID, block height, and difficulty are sent to web dashboard.
//   - Compatible with miner_stats.rs v1.0.9 and web_server.rs v1.0.1.
// - v2.0.1-web (2025-06-22): Added web dashboard support.
//   - Added get_stats() method to expose MinerStats for web dashboard.
//   - Returns Arc<MinerStats> for real-time statistics access.
//   - Maintains all existing SHA3x mining and SV2 testing functionality.
//   - Compatible with web_server.rs for real-time dashboard integration.
// - v2.0.0-sv2-test: Complete rewrite for SV2 testing
//   - Removed all SHA-256/Bitcoin Stratum V1 code
//   - Added basic SV2 TCP connection testing
//   - Preserved SHA3x functionality unchanged
//   - Added test_sv2_connection() method for JDS testing