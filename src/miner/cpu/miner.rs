// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/miner/cpu/miner.rs
// Version: 1.0.4
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file contains the main CPU miner implementation for the SHA3x miner,
// located in the cpu subdirectory of the miner module. It coordinates pool
// communication, mining threads, and share submission.
//
// Tree Location:
// - src/miner/cpu/miner.rs (main CPU miner logic)
// - Depends on: core, pool, stats, thread, tokio, num_cpus, rand

use crate::core::{parse_target_difficulty, PoolJob, MiningJob};
use crate::miner::stats::MinerStats;
use crate::miner::cpu::thread::start_mining_thread;
use crate::pool::PoolClient;
use crate::Result;
use num_cpus;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use tokio::sync::mpsc::{self, UnboundedReceiver as MpscReceiver};
use tokio::sync::broadcast::{self, Sender as BroadcastSender};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct CpuMiner {
    wallet_address: String,
    pool_address: SocketAddr,
    worker_name: String,
    num_threads: usize,
    stats: Arc<MinerStats>,
    pool_client: PoolClient,
}

impl CpuMiner {
    pub fn new(
        wallet_address: String,
        pool_address: SocketAddr,
        worker_name: String,
        num_threads: usize,
    ) -> Self {
        let actual_threads = if num_threads == 0 {
            num_cpus::get()
        } else {
            num_threads
        };

        Self {
            wallet_address,
            pool_address,
            worker_name,
            num_threads: actual_threads,
            stats: Arc::new(MinerStats::new(actual_threads)),
            pool_client: PoolClient::new(),
        }
    }

    async fn connect_to_pool(&self) -> Result<tokio::net::TcpStream> {
        Ok(self.pool_client.connect(self.pool_address).await?)
    }

    async fn login(&self, writer: &mut tokio::net::tcp::OwnedWriteHalf) -> Result<()> {
        let login_request = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "login",
            "params": {
                "login": self.wallet_address,
                "pass": self.worker_name,
                "agent": "sha3x-miner/3.0",
                "algo": "sha3x"
            }
        });

        let message = format!("{}\n", login_request);
        writer.write_all(message.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn handle_pool_message(
        &self,
        message: &str,
        job_tx: &BroadcastSender<MiningJob>,
    ) -> Result<()> {
        debug!("📨 Pool message: {}", message);
        let response: Value = serde_json::from_str(message)?;

        if let Some(method) = response.get("method").and_then(|m| m.as_str()) {
            if method == "job" {
                debug!("Processing job message: {:?}", response);
                if let Some(params) = response.get("params").and_then(|p| p.as_object()) {
                    self.handle_new_job(params, job_tx).await?;
                    if let Some(diff) = params.get("difficulty").and_then(|d| d.as_u64()) {
                        self.stats.add_activity(format!("🔧 VarDiff update: {}", MinerStats::format_number(diff)));
                        info!("🔧 VarDiff job update received");
                    }
                }
            }
        } else if let Some(result) = response.get("result") {
            debug!("Result response: {:?}", result);
            if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
                if id == 1 {
                    info!("✅ Login successful");
                    self.stats.add_activity("🔐 Connected successfully".to_string());
                    // Check if result contains a job
                    if let Some(job_params) = result.get("job").and_then(|j| j.as_object()) {
                        debug!("Found job in login response: {:?}", job_params);
                        self.handle_new_job(job_params, job_tx).await?;
                    }
                } else if id >= 100 {
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
            }
        } else if let Some(error) = response.get("error") {
            error!("❌ Pool error: {:?}", error);
            self.stats.add_activity(format!("🚫 Pool error: {}", message));
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
        let header_template = hex::decode(&job.blob)?;
        let target_difficulty = job.difficulty.unwrap_or_else(|| parse_target_difficulty(&job.target));

        let mining_job = MiningJob {
            job_id: job.job_id.clone(),
            mining_hash: header_template,
            target_difficulty,
            height: job.height,
        };

        job_tx.send(mining_job)?;
        info!("📋 New job: {} (height: {}, difficulty: {})", 
            job.job_id, job.height, MinerStats::format_number(target_difficulty));
        self.stats.add_activity(format!(
            "📋 New job: {} (height: {}, difficulty: {})",
            &job.job_id[..8], job.height, MinerStats::format_number(target_difficulty)
        ));

        Ok(())
    }

    fn start_share_submitter(
        &self,
        writer: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
        mut share_rx: MpscReceiver<(String, String, String, usize, u64)>,
    ) {
        let wallet_address = self.wallet_address.clone();
        let num_threads = self.num_threads;

        tokio::spawn(async move {
            let mut submit_id = 100;

            while let Some((job_id, nonce, result, thread_id, _difficulty)) = share_rx.recv().await {
                let submit_request = json!({
                    "id": submit_id + thread_id as u64,
                    "jsonrpc": "2.0",
                    "method": "submit",
                    "params": {
                        "id": wallet_address.clone(),
                        "job_id": job_id,
                        "nonce": nonce,
                        "result": result
                    }
                });

                info!("📤 Submitting share: {}", submit_request);
                info!("📤 Details - job_id: {}, nonce: {}, result: {}", job_id, nonce, result);

                let message = format!("{}\n", submit_request);
                let mut writer = writer.lock().await;
                if let Err(e) = writer.write_all(message.as_bytes()).await {
                    error!("Failed to submit share: {}", e);
                }

                submit_id += num_threads as u64;
            }
        });
    }

    fn start_stats_printer(&self) {
        let stats = Arc::clone(&self.stats);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                // Generate 16-char hex ID per iteration using u64
                let dashboard_id = format!("{:016x}", thread_rng().r#gen::<u64>());
                stats.display_dashboard(&dashboard_id);
            }
        });
    }

    pub async fn run(&self) -> Result<()> {
        let stream = self.connect_to_pool().await?;
        info!("✅ Connected to pool");
        self.stats.add_activity("🔐 Connected to pool".to_string());

        let (reader, writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(writer));

        self.login(&mut *writer.lock().await).await?;
        info!("🔐 Login request sent");

        let (job_tx, _) = broadcast::channel(16);
        let (share_tx, share_rx) = mpsc::unbounded_channel();

        for thread_id in 0..self.num_threads {
            let job_rx = job_tx.subscribe();
            let share_tx = share_tx.clone();
            let thread_stats = Arc::clone(&self.stats.thread_stats[thread_id]);
            let stats = Arc::clone(&self.stats);
            start_mining_thread(thread_id, self.num_threads, job_rx, share_tx, thread_stats, stats);
        }

        self.start_share_submitter(Arc::clone(&writer), share_rx);
        self.start_stats_printer();

        let reader = BufReader::new(reader);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            self.handle_pool_message(&line, &job_tx).await?;
        }

        Ok(())
    }
}

// Changelog:
// - v1.0.4 (2025-06-14T02:38:00Z): Enhanced share response logging.
//   - Added debug logging in handle_pool_message to capture all share responses and errors.
//   - Fixed compilation issues from u32 usage, using u64 for dashboard ID.
//   - Maintained original share submission and pool message handling logic.
// - v1.0.3 (2025-06-14T01:34:00Z): Added dashboard display.
//   - Updated start_stats_printer to call display_dashboard.
// - v1.0.2 (2025-06-13T23:52:00Z): Restored tokio::sync::mpsc channels.
// - v1.0.1 (2025-06-13T23:51:00Z): Fixed compilation issues.
// - v1.0.0 (2025-06-14T00:00:00Z): Extracted from monolithic main.rs.