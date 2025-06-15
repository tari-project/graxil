// SHA3x Tari Miner v3.0 - Complete TUI Edition - Part 1
// Cargo.toml dependencies:
// [dependencies]
// sha3 = "0.10"
// tokio = { version = "1.0", features = ["full"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// hex = "0.4"
// clap = { version = "4.0", features = ["derive"] }
// tracing = "0.1"
// tracing-subscriber = "0.3"
// crossbeam = "0.8"
// num_cpus = "1.0"
// rand = "0.8"
// ratatui = "0.28"
// crossterm = "0.28"

use clap::Parser;
use crossbeam::channel;
use hex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};
use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info};

// Ratatui imports
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Tari wallet address
    #[arg(short, long)]
    wallet: String,

    /// Mining pool address (host:port)
    #[arg(short, long)]
    pool: String,

    /// Worker name
    #[arg(long, default_value = "worker1")]
    worker: String,

    /// Number of CPU threads (0 = auto)
    #[arg(short, long, default_value = "0")]
    threads: usize,

    /// Enable GPU mining (placeholder)
    #[arg(short, long, default_value = "false")]
    gpu: bool,

    /// Enable TUI dashboard
    #[arg(long, default_value = "false")]
    tui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Job {
    blob: String,
    job_id: String,
    target: String,
    algo: String,
    height: u64,
    seed_hash: Option<String>,
    #[serde(default)]
    difficulty: Option<u64>,
}

#[derive(Debug, Clone)]
struct MiningJob {
    job_id: String,
    header_template: Vec<u8>,
    target_difficulty: u64,
    height: u64,
}

#[derive(Debug)]
struct ThreadStats {
    thread_id: usize,
    hashes_computed: AtomicU64,
    shares_found: AtomicU64,
    shares_rejected: AtomicU64,
    last_share_time: Arc<Mutex<Option<Instant>>>,
    start_time: Instant,
    current_hashrate: Arc<Mutex<f64>>,
    best_difficulty: AtomicU64,
    current_difficulty_target: AtomicU64,
}

impl ThreadStats {
    fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            hashes_computed: AtomicU64::new(0),
            shares_found: AtomicU64::new(0),
            shares_rejected: AtomicU64::new(0),
            last_share_time: Arc::new(Mutex::new(None)),
            start_time: Instant::now(),
            current_hashrate: Arc::new(Mutex::new(0.0)),
            best_difficulty: AtomicU64::new(0),
            current_difficulty_target: AtomicU64::new(0),
        }
    }

    fn record_share(&self, difficulty: u64, accepted: bool) {
        if accepted {
            self.shares_found.fetch_add(1, Ordering::Relaxed);
        } else {
            self.shares_rejected.fetch_add(1, Ordering::Relaxed);
        }
        
        *self.last_share_time.lock().unwrap() = Some(Instant::now());
        
        let current_best = self.best_difficulty.load(Ordering::Relaxed);
        if difficulty > current_best {
            self.best_difficulty.store(difficulty, Ordering::Relaxed);
        }
    }

    fn update_hashrate(&self, hashes: u64) {
        self.hashes_computed.fetch_add(hashes, Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
            *self.current_hashrate.lock().unwrap() = total_hashes as f64 / elapsed;
        }
    }

    fn get_hashrate(&self) -> f64 {
        *self.current_hashrate.lock().unwrap()
    }

    fn get_share_dots(&self) -> String {
        let accepted = self.shares_found.load(Ordering::Relaxed);
        let rejected = self.shares_rejected.load(Ordering::Relaxed);
        
        let mut dots = String::new();
        for _ in 0..accepted.min(5) {
            dots.push('●');
        }
        for _ in 0..rejected.min(5) {
            dots.push('●'); // Will be colored red in display
        }
        dots
    }
}

#[derive(Debug)]
struct ShareInfo {
    time: Instant,
    thread_id: usize,
    difficulty: u64,
    target: u64,
    accepted: bool,
}

#[derive(Debug)]
struct MinerStats {
    shares_submitted: AtomicU64,
    shares_accepted: AtomicU64,
    shares_rejected: AtomicU64,
    hashes_computed: AtomicU64,
    start_time: Instant,
    thread_stats: Vec<Arc<ThreadStats>>,
    recent_shares: Arc<Mutex<VecDeque<ShareInfo>>>,
    recent_activity: Arc<Mutex<VecDeque<(Instant, String)>>>,
    hashrate_history: Arc<Mutex<VecDeque<(Instant, u64)>>>,
}

impl MinerStats {
    fn new(num_threads: usize) -> Self {
        let mut thread_stats = Vec::new();
        for i in 0..num_threads {
            thread_stats.push(Arc::new(ThreadStats::new(i)));
        }

        Self {
            shares_submitted: AtomicU64::new(0),
            shares_accepted: AtomicU64::new(0),
            shares_rejected: AtomicU64::new(0),
            hashes_computed: AtomicU64::new(0),
            start_time: Instant::now(),
            thread_stats,
            recent_shares: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            recent_activity: Arc::new(Mutex::new(VecDeque::with_capacity(50))),
            hashrate_history: Arc::new(Mutex::new(VecDeque::with_capacity(300))),
        }
    }

    fn add_activity(&self, message: String) {
        let mut activity = self.recent_activity.lock().unwrap();
        activity.push_back((Instant::now(), message));
        if activity.len() > 50 {
            activity.pop_front();
        }
    }

    fn record_share_found(&self, thread_id: usize, difficulty: u64, target: u64, accepted: bool) {
        if thread_id < self.thread_stats.len() {
            self.thread_stats[thread_id].record_share(difficulty, accepted);
        }
        
        // Add to recent shares
        let mut shares = self.recent_shares.lock().unwrap();
        shares.push_back(ShareInfo {
            time: Instant::now(),
            thread_id,
            difficulty,
            target,
            accepted,
        });
        if shares.len() > 100 {
            shares.pop_front();
        }
    }

    fn update_hashrate_history(&self, total_hashes: u64) {
        let mut history = self.hashrate_history.lock().unwrap();
        history.push_back((Instant::now(), total_hashes));
        
        // Keep only last 5 minutes
        let cutoff = Instant::now() - Duration::from_secs(300);
        while let Some((time, _)) = history.front() {
            if *time < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_total_hashrate(&self) -> f64 {
        let total_hashes = self.hashes_computed.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            total_hashes as f64 / elapsed
        } else {
            0.0
        }
    }

    fn get_active_thread_count(&self) -> usize {
        self.thread_stats
            .iter()
            .filter(|t| t.get_hashrate() > 0.0)
            .count()
    }

    fn get_avg_hashrate_per_thread(&self) -> f64 {
        let active = self.get_active_thread_count();
        if active > 0 {
            self.get_total_hashrate() / active as f64
        } else {
            0.0
        }
    }

    fn get_share_rate_per_minute(&self) -> f64 {
        let shares = self.shares_submitted.load(Ordering::Relaxed);
        let elapsed_mins = self.start_time.elapsed().as_secs_f64() / 60.0;
        if elapsed_mins > 0.0 {
            shares as f64 / elapsed_mins
        } else {
            0.0
        }
    }

    fn format_hashrate(hashrate: f64) -> String {
        if hashrate >= 1_000_000_000.0 {
            format!("{:.2} GH/s", hashrate / 1_000_000_000.0)
        } else if hashrate >= 1_000_000.0 {
            format!("{:.2} MH/s", hashrate / 1_000_000.0)
        } else if hashrate >= 1_000.0 {
            format!("{:.2} KH/s", hashrate / 1_000.0)
        } else {
            format!("{:.2} H/s", hashrate)
        }
    }

    fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else {
            format!("{}h ago", secs / 3600)
        }
    }

    fn format_number(num: u64) -> String {
        if num >= 1_000_000_000 {
            format!("{:.1}B", num as f64 / 1_000_000_000.0)
        } else if num >= 1_000_000 {
            format!("{:.1}M", num as f64 / 1_000_000.0)
        } else if num >= 1_000 {
            format!("{:.1}K", num as f64 / 1_000.0)
        } else {
            num.to_string()
        }
    }
}

struct SHA3xMiner {
    wallet_address: String,
    pool_address: SocketAddr,
    worker_name: String,
    num_threads: usize,
    stats: Arc<MinerStats>,
    enable_tui: bool,
}

// End of Part 1

// SHA3x Tari Miner v3.0 - Complete TUI Edition - Part 2
// Continue from Part 1

impl SHA3xMiner {
    fn new(
        wallet_address: String,
        pool_address: SocketAddr,
        worker_name: String,
        num_threads: usize,
        enable_tui: bool,
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
            enable_tui,
        }
    }

    async fn connect_to_pool(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        Ok(TcpStream::connect(self.pool_address).await?)
    }

    async fn login(&self, writer: &mut tokio::net::tcp::OwnedWriteHalf) -> Result<(), Box<dyn std::error::Error>> {
        let login_request = serde_json::json!({
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
        job_tx: &broadcast::Sender<MiningJob>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("📨 Pool message: {}", message);
        let response: Value = serde_json::from_str(message)?;

        if let Some(method) = response.get("method").and_then(|m| m.as_str()) {
            match method {
                "job" => {
                    if let Some(params) = response.get("params").and_then(|p| p.as_object()) {
                        self.handle_new_job(params, job_tx).await?;
                        
                        // Check for VarDiff update
                        if let Some(diff) = params.get("difficulty").and_then(|d| d.as_u64()) {
                            self.stats.add_activity(format!("🔧 VarDiff update: {}", MinerStats::format_number(diff)));
                            info!("🔧 VarDiff job update received");
                        }
                    }
                }
                _ => {}
            }
        } else if let Some(result) = response.get("result") {
            if let Some(id) = response.get("id").and_then(|id| id.as_u64()) {
                if id == 1 {
                    info!("✅ Login successful");
                    self.stats.add_activity("🔐 Connected successfully".to_string());
                } else if id >= 100 {
                    // Share submission response
                    let thread_id = (id - 100) as usize % self.num_threads;
                    
                    // Check different possible response formats
                    let accepted = if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
                        match status.to_lowercase().as_str() {
                            "ok" | "accepted" => true,
                            _ => {
                                error!("❌ Share rejected with status: {} (full result: {:?})", status, result);
                                false
                            }
                        }
                    } else if result.is_null() {
                        // Some pools return null for accepted shares
                        info!("✅ Share accepted (null response)");
                        true
                    } else if let Some(accepted) = result.as_bool() {
                        // Some pools return boolean
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
                        self.stats.add_activity(format!("❌ Share rejected from thread {}", thread_id));
                    }
                    
                    // Update thread stats
                    if thread_id < self.stats.thread_stats.len() {
                        self.stats.thread_stats[thread_id].record_share(0, accepted);
                    }
                }
            }
        } else if response.get("error").is_some() {
            error!("❌ Pool error: {}", message);
            self.stats.add_activity(format!("🚫 Pool error: {}", message));
        }

        Ok(())
    }

    async fn handle_new_job(
        &self,
        job_data: &serde_json::Map<String, Value>,
        job_tx: &broadcast::Sender<MiningJob>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let job: Job = serde_json::from_value(Value::Object(job_data.clone()))?;
        
        let header_template = hex::decode(&job.blob)?;
        
        let target_difficulty = if let Some(diff) = job.difficulty {
            diff
        } else {
            self.parse_target_difficulty(&job.target)
        };
        
        let mining_job = MiningJob {
            job_id: job.job_id.clone(),
            header_template,
            target_difficulty,
            height: job.height,
        };
        
        let _ = job_tx.send(mining_job);
        
        info!("📋 New job: {} (height: {}, difficulty: {})", 
            job.job_id, job.height, MinerStats::format_number(target_difficulty));
        
        self.stats.add_activity(format!(
            "📋 New job: {} (height: {}, difficulty: {})",
            &job.job_id[..8],
            job.height,
            MinerStats::format_number(target_difficulty)
        ));
        
        Ok(())
    }

    fn parse_target_difficulty(&self, target_hex: &str) -> u64 {
        match hex::decode(target_hex) {
            Ok(target_bytes) => {
                if target_bytes.len() >= 8 {
                    let target_u64 = u64::from_le_bytes([
                        target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3],
                        target_bytes[4], target_bytes[5], target_bytes[6], target_bytes[7],
                    ]);
                    
                    if target_u64 > 0 {
                        0xFFFFFFFFFFFFFFFFu64 / target_u64
                    } else {
                        1
                    }
                } else {
                    1
                }
            }
            Err(_) => 1,
        }
    }

    fn sha3x_hash_with_nonce(header_template: &[u8], nonce: u64) -> Vec<u8> {
        let mut input = Vec::with_capacity(header_template.len() + 9);
        input.extend_from_slice(&nonce.to_le_bytes());
        input.extend_from_slice(header_template);
        input.push(1u8);
        
        let hash1 = Sha3_256::digest(&input);
        let hash2 = Sha3_256::digest(&hash1);
        let hash3 = Sha3_256::digest(&hash2);
        
        hash3.to_vec()
    }

    fn calculate_difficulty(hash: &[u8]) -> u64 {
        if hash.len() < 8 {
            return 0;
        }
        
        let hash_u64 = u64::from_be_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ]);
        
        if hash_u64 == 0 {
            return u64::MAX;
        }
        
        0xFFFFFFFFFFFFFFFFu64 / hash_u64
    }

    fn start_mining_threads(
        &self,
        job_tx: broadcast::Sender<MiningJob>,
        share_tx: mpsc::UnboundedSender<(String, String, String, usize, u64)>,
    ) {
        let num_threads = self.num_threads;
        let stats = Arc::clone(&self.stats);
        let should_stop = Arc::new(AtomicBool::new(false));
        
        info!("⚡ Starting {} mining threads", num_threads);
        
        for thread_id in 0..num_threads {
            let job_rx = job_tx.subscribe();
            let share_tx = share_tx.clone();
            let thread_stats = Arc::clone(&stats.thread_stats[thread_id]);
            let stats = Arc::clone(&stats);
            let should_stop = Arc::clone(&should_stop);
            
            std::thread::spawn(move || {
                Self::mining_thread(thread_id, num_threads, job_rx, share_tx, thread_stats, stats, should_stop);
            });
        }
    }

    fn mining_thread(
        thread_id: usize,
        num_threads: usize,
        mut job_rx: broadcast::Receiver<MiningJob>,
        share_tx: mpsc::UnboundedSender<(String, String, String, usize, u64)>,
        thread_stats: Arc<ThreadStats>,
        stats: Arc<MinerStats>,
        should_stop: Arc<AtomicBool>,
    ) {
        let mut rng = rand::rng();
        let mut current_job: Option<MiningJob> = None;
        let mut hash_count = 0u64;
        let mut last_report = Instant::now();
        
        loop {
            if should_stop.load(Ordering::Relaxed) {
                break;
            }
            
            match job_rx.try_recv() {
                Ok(job) => {
                    thread_stats.current_difficulty_target.store(job.target_difficulty, Ordering::Relaxed);
                    current_job = Some(job);
                }
                Err(broadcast::error::TryRecvError::Empty) => {}
                Err(_) => break,
            }
            
            if let Some(ref job) = current_job {
                let mut nonce = rng.random::<u64>();
                nonce = nonce.wrapping_add(thread_id as u64);
                
                for _ in 0..10000 {
                    let hash = Self::sha3x_hash_with_nonce(&job.header_template, nonce);
                    let difficulty = Self::calculate_difficulty(&hash);
                    hash_count += 1;
                    
                    if difficulty >= job.target_difficulty {
                        // Try both endianness for nonce
                        let nonce_hex = hex::encode(nonce.to_le_bytes());
                        let nonce_hex_be = hex::encode(nonce.to_be_bytes());
                        let result_hex = hex::encode(&hash);
                        
                        thread_stats.record_share(difficulty, true);
                        stats.record_share_found(thread_id, difficulty, job.target_difficulty, true);
                        
                        info!("💎 Thread {} found share! Difficulty: {}, Target: {}", 
                            thread_id, 
                            MinerStats::format_number(difficulty), 
                            MinerStats::format_number(job.target_difficulty));
                        
                        info!("🔍 Share details - Nonce LE: {}, Nonce BE: {}", nonce_hex, nonce_hex_be);
                        info!("🔍 Hash result: {}", result_hex);
                        
                        stats.add_activity(format!(
                            "💎 Thread {} found share! Difficulty: {}",
                            thread_id,
                            MinerStats::format_number(difficulty)
                        ));
                        
                        let _ = share_tx.send((
                            job.job_id.clone(),
                            nonce_hex_be,  // Try BE instead of LE
                            result_hex,
                            thread_id,
                            difficulty,
                        ));
                        
                        stats.shares_submitted.fetch_add(1, Ordering::Relaxed);
                    }
                    
                    nonce = nonce.wrapping_add(num_threads as u64);
                }
                
                if last_report.elapsed() > Duration::from_secs(1) {
                    thread_stats.update_hashrate(hash_count);
                    stats.hashes_computed.fetch_add(hash_count, Ordering::Relaxed);
                    stats.update_hashrate_history(stats.hashes_computed.load(Ordering::Relaxed));
                    hash_count = 0;
                    last_report = Instant::now();
                }
            } else {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    fn start_share_submitter(
        &self,
        writer: Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>,
        mut share_rx: mpsc::UnboundedReceiver<(String, String, String, usize, u64)>,
    ) {
        let wallet_address = self.wallet_address.clone();
        let num_threads = self.num_threads;
        
        tokio::spawn(async move {
            let mut submit_id = 100;
            
            while let Some((job_id, nonce, result, thread_id, _difficulty)) = share_rx.recv().await {
                let submit_request = serde_json::json!({
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
        if self.enable_tui {
            return;
        }
        
        let stats = Arc::clone(&self.stats);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let total_hashes = stats.hashes_computed.load(Ordering::Relaxed);
                let shares_submitted = stats.shares_submitted.load(Ordering::Relaxed);
                let shares_accepted = stats.shares_accepted.load(Ordering::Relaxed);
                
                let acceptance_rate = if shares_submitted > 0 {
                    (shares_accepted as f64 / shares_submitted as f64) * 100.0
                } else {
                    0.0
                };
                
                info!(
                    "📊 Stats: {} | Shares: {}/{} ({:.1}%) | Hashes: {}",
                    MinerStats::format_hashrate(stats.get_total_hashrate()),
                    shares_accepted,
                    shares_submitted,
                    acceptance_rate,
                    total_hashes
                );
            }
        });
    }

    async fn run_tui(&self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let mut current_tab = 0;
        
        loop {
            terminal.draw(|f| {
                self.draw_ui(f, current_tab);
            })?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Tab | KeyCode::Right => {
                            current_tab = (current_tab + 1) % 3;
                        }
                        KeyCode::Left => {
                            current_tab = if current_tab == 0 { 2 } else { current_tab - 1 };
                        }
                        _ => {}
                    }
                }
            }
        }
        
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(())
    }

    fn draw_ui(&self, f: &mut Frame, current_tab: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.area());
        
        let titles = vec!["Overview", "Threads", "Shares"];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("SHA3x Miner Dashboard"))
            .select(current_tab)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, chunks[0]);
        
        match current_tab {
            0 => self.draw_overview(f, chunks[1]),
            1 => self.draw_threads(f, chunks[1]),
            2 => self.draw_shares(f, chunks[1]),
            _ => {}
        }
    }

    fn draw_overview(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Min(0),
            ].as_ref())
            .split(area);
        
        // Pool connection info
        let pool_info = vec![
            Line::from(vec![
                Span::styled("Pool: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(self.pool_address.to_string(), Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled("Worker: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(&self.worker_name, Style::default().fg(Color::Green)),
            ]),
        ];
        let pool_block = Paragraph::new(pool_info)
            .block(Block::default().borders(Borders::ALL).title("Connection"));
        f.render_widget(pool_block, chunks[0]);
        
        // Mining statistics
        let total_hashrate = self.stats.get_total_hashrate();
        let shares_submitted = self.stats.shares_submitted.load(Ordering::Relaxed);
        let shares_accepted = self.stats.shares_accepted.load(Ordering::Relaxed);
        let shares_rejected = self.stats.shares_rejected.load(Ordering::Relaxed);
        
        let acceptance_rate = if shares_submitted > 0 {
            (shares_accepted as f64 / shares_submitted as f64) * 100.0
        } else {
            0.0
        };
        
        let acceptance_color = if acceptance_rate >= 95.0 {
            Color::Green
        } else if acceptance_rate >= 85.0 {
            Color::Yellow  
        } else {
            Color::Red
        };
        
        let rejected_color = if shares_rejected == 0 {
            Color::Green
        } else {
            Color::Red
        };
        
        let uptime = self.stats.start_time.elapsed();
        let current_diff = if let Some(share) = self.stats.recent_shares.lock().unwrap().back() {
            share.target
        } else {
            0
        };
        
        let stats_text = vec![
            Line::from(vec![
                Span::styled("Hashrate: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_hashrate(total_hashrate), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled("Shares: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}/{}", shares_accepted, shares_submitted), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(format!("({:.1}%)", acceptance_rate), Style::default().fg(acceptance_color)),
            ]),
            Line::from(vec![
                Span::styled("Accepted: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(shares_accepted.to_string(), Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled("Rejected: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(shares_rejected.to_string(), Style::default().fg(rejected_color)),
                Span::raw("  "),
                Span::styled("Difficulty: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_number(current_diff), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Uptime: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}h {}m {}s", uptime.as_secs() / 3600, (uptime.as_secs() % 3600) / 60, uptime.as_secs() % 60), Style::default().fg(Color::Blue)),
                Span::raw("  "),
                Span::styled("Threads: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}/{}", self.stats.get_active_thread_count(), self.num_threads), Style::default().fg(Color::Cyan)),
            ]),
        ];
        
        let stats_block = Paragraph::new(stats_text)
            .block(Block::default().borders(Borders::ALL).title("Mining Statistics"));
        f.render_widget(stats_block, chunks[1]);
        
        // Worker info
        let wallet_short = if self.wallet_address.len() > 40 {
            format!("{}...{}", &self.wallet_address[..20], &self.wallet_address[self.wallet_address.len()-20..])
        } else {
            self.wallet_address.clone()
        };
        
        let worker_info = vec![
            Line::from(vec![
                Span::styled("Wallet: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(wallet_short, Style::default().fg(Color::Cyan)),
            ]),
        ];
        let worker_block = Paragraph::new(worker_info)
            .block(Block::default().borders(Borders::ALL).title("Worker"));
        f.render_widget(worker_block, chunks[2]);
        
        // Recent activity
        let mut activity_lines = vec![];
        let activities = self.stats.recent_activity.lock().unwrap();
        for (time, message) in activities.iter().rev().take(20) {
            let elapsed = time.elapsed();
            let time_str = format!("{:>3}s", elapsed.as_secs());
            activity_lines.push(Line::from(vec![
                Span::styled(time_str, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::raw(message),
            ]));
        }
        
        let activity_block = Paragraph::new(activity_lines)
            .block(Block::default().borders(Borders::ALL).title("Recent Activity"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(activity_block, chunks[3]);
    }

    fn draw_threads(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
            ].as_ref())
            .split(area);
        
        self.draw_thread_summary(f, chunks[0]);
        self.draw_thread_grid(f, chunks[1]);
    }

    fn draw_thread_summary(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        
        // Left side - Stats and hashrate graph
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(chunks[0]);
        
        // Summary stats
        let total_hashrate = self.stats.get_total_hashrate();
        let active_threads = self.stats.get_active_thread_count();
        let avg_per_thread = self.stats.get_avg_hashrate_per_thread();
        let total_shares = self.stats.shares_submitted.load(Ordering::Relaxed);
        let share_rate = self.stats.get_share_rate_per_minute();
        let last_share_time = self.stats.recent_shares.lock().unwrap()
            .back()
            .map(|s| s.time.elapsed())
            .unwrap_or(Duration::from_secs(0));
        
        let summary_text = vec![
            Line::from(vec![
                Span::styled("Total Hashrate: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_hashrate(total_hashrate), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled("Active Threads: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}/{}", active_threads, self.num_threads), Style::default().fg(Color::Cyan)),
                Span::raw("    "),
                Span::styled("Avg per Thread: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_hashrate(avg_per_thread), Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("Total Shares: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(total_shares.to_string(), Style::default().fg(Color::Green)),
                Span::raw("    "),
                Span::styled("Recent Activity: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_duration(last_share_time), Style::default().fg(Color::Cyan)),
                Span::raw("    "),
                Span::styled("Share Rate: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:.1}/min", share_rate), Style::default().fg(Color::Magenta)),
            ]),
        ];
        
        let summary_block = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary Stats"));
        f.render_widget(summary_block, left_chunks[0]);
        
        // Hashrate graph
        let graph_block = Block::default()
            .borders(Borders::ALL)
            .title("Hashrate Graph (Last 5 min)");
        f.render_widget(graph_block, left_chunks[1]);
        
        let graph_text = self.create_hashrate_graph();
        let graph = Paragraph::new(graph_text);
        f.render_widget(graph, left_chunks[1]);
        
        // Right side - Share timeline
        let share_timeline_block = Block::default()
            .borders(Borders::ALL)
            .title("Share Timeline (Last 10 shares)");
        f.render_widget(share_timeline_block, chunks[1]);
        
        let timeline_text = self.create_share_timeline();
        let timeline = Paragraph::new(timeline_text);
        f.render_widget(timeline, chunks[1]);
    }

    fn draw_thread_grid(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Active Threads (24x3 Dynamic Grid)");
        
        let inner_area = block.inner(area);
        f.render_widget(block, area);
        
        // Calculate grid dimensions
        let cols_per_row = 24;
        let cell_height = 4;
        
        // Get active threads
        let mut active_threads = Vec::new();
        for (idx, thread_stat) in self.stats.thread_stats.iter().enumerate() {
            if thread_stat.get_hashrate() > 0.0 {
                active_threads.push((idx, thread_stat));
            }
        }
        
        // Create grid lines
        let mut row_lines = vec![String::new(); cell_height];
        let mut current_col = 0;
        let mut grid_lines = vec![];
        
        for (idx, thread_stat) in &active_threads {
            let hashrate = thread_stat.get_hashrate();
            let hashrate_str = if hashrate >= 1_000_000.0 {
                format!("{:.0}M", hashrate / 1_000_000.0)
            } else if hashrate >= 1_000.0 {
                format!("{:.0}K", hashrate / 1_000.0)
            } else {
                format!("{:.0}", hashrate)
            };
            
            let share_dots = thread_stat.get_share_dots();
            let accepted = thread_stat.shares_found.load(Ordering::Relaxed);
            let rejected = thread_stat.shares_rejected.load(Ordering::Relaxed);
            
            let last_share = thread_stat.last_share_time.lock().unwrap()
                .map(|t| MinerStats::format_duration(t.elapsed()))
                .unwrap_or_else(|| "---".to_string());
            
            // Build cell
            row_lines[0] += &format!("┌──────");
            row_lines[1] += &format!("│{:^6}", format!("{:02}", idx));
            row_lines[2] += &format!("│{:>5} ", hashrate_str);
            
            // Add share dots with proper coloring logic
            if !share_dots.is_empty() {
                let mut dot_str = String::new();
                for _ in 0..accepted.min(5) {
                    dot_str.push('●'); // Green dots
                }
                for _ in 0..rejected.min(5) {
                    dot_str.push('○'); // Red dots (will be colored in terminal)
                }
                row_lines[2] += &format!("{:<6}", dot_str);
            } else {
                row_lines[2] += &format!("{:<6}", "");
            }
            
            row_lines[3] += &format!("│{:>5} ", &last_share[..last_share.len().min(5)]);
            
            current_col += 1;
            
            // Start new row if needed
            if current_col >= cols_per_row || idx == &active_threads.last().unwrap().0 {
                // Close the row
                for i in 0..cell_height {
                    if i == 0 {
                        row_lines[i] += "┐";
                    } else {
                        row_lines[i] += "│";
                    }
                }
                
                // Add to grid lines
                for line in &row_lines {
                    grid_lines.push(Line::from(line.clone()));
                }
                
                // Add bottom border if not last row
                if idx != &active_threads.last().unwrap().0 {
                    let mut bottom_line = String::new();
                    for _ in 0..current_col {
                        bottom_line += "└──────";
                    }
                    bottom_line += "┘";
                    grid_lines.push(Line::from(bottom_line));
                }
                
                // Reset for next row
                row_lines = vec![String::new(); cell_height];
                current_col = 0;
            }
        }
        
        // Add final bottom border
        if !grid_lines.is_empty() && current_col > 0 {
            let mut bottom_line = String::new();
            for _ in 0..current_col {
                bottom_line += "└──────";
            }
            bottom_line += "┘";
            grid_lines.push(Line::from(bottom_line));
        }
        
        // Add legend
        grid_lines.push(Line::from(""));
        grid_lines.push(Line::from(vec![
            Span::raw("Legend: "),
            Span::styled("● Good Share", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("○ Rejected", Style::default().fg(Color::Red)),
            Span::raw(" | Time = Last Share Age"),
        ]));
        
        let grid_paragraph = Paragraph::new(grid_lines);
        f.render_widget(grid_paragraph, inner_area);
    }

    fn draw_shares(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
            ].as_ref())
            .split(area);
        
        self.draw_share_summary(f, chunks[0]);
        
        // Recent shares with details
        let shares_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(chunks[1]);
        
        // Recent shares list
        let mut share_lines = vec![];
        let shares = self.stats.recent_shares.lock().unwrap();
        
        for share in shares.iter().rev().take(20) {
            let luck = share.difficulty as f64 / share.target as f64;
            let luck_indicator = if luck >= 10.0 {
                " 🎰"
            } else if luck >= 5.0 {
                " 🍀"
            } else if luck >= 2.0 {
                " 🎲"
            } else {
                ""
            };
            
            let status_color = if share.accepted {
                Color::Green
            } else {
                Color::Red
            };
            
            let status_symbol = if share.accepted { "●" } else { "○" };
            
            share_lines.push(Line::from(vec![
                Span::styled(MinerStats::format_duration(share.time.elapsed()), Style::default().fg(Color::DarkGray)),
                Span::raw(": "),
                Span::styled(format!("Thread {:02}", share.thread_id), Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::styled(MinerStats::format_number(share.difficulty), Style::default().fg(Color::Yellow)),
                Span::raw(" (target: "),
                Span::styled(MinerStats::format_number(share.target), Style::default().fg(Color::Magenta)),
                Span::raw(") - "),
                Span::styled(format!("{:.2}x luck", luck), Style::default().fg(Color::Blue)),
                Span::raw(" "),
                Span::styled(status_symbol, Style::default().fg(status_color)),
                Span::raw(luck_indicator),
            ]));
        }
        
        let shares_block = Paragraph::new(share_lines)
            .block(Block::default().borders(Borders::ALL).title("Recent Shares (Luck Analysis)"));
        f.render_widget(shares_block, shares_chunks[0]);
        
        // Thread performance leaders
        let mut thread_shares: Vec<(usize, u64, u64)> = Vec::new();
        for (idx, thread) in self.stats.thread_stats.iter().enumerate() {
            let accepted = thread.shares_found.load(Ordering::Relaxed);
            let rejected = thread.shares_rejected.load(Ordering::Relaxed);
            if accepted > 0 || rejected > 0 {
                thread_shares.push((idx, accepted, rejected));
            }
        }
        thread_shares.sort_by(|a, b| b.1.cmp(&a.1));
        
        let total_shares = self.stats.shares_submitted.load(Ordering::Relaxed);
        let mut leader_lines = vec![
            Line::from(vec![
                Span::styled("Thread Performance Leaders", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];
        
        for (i, (thread_id, accepted, _rejected)) in thread_shares.iter().take(5).enumerate() {
            let percentage = if total_shares > 0 {
                (*accepted as f64 / total_shares as f64) * 100.0
            } else {
                0.0
            };
            
            leader_lines.push(Line::from(vec![
                Span::raw(format!("{}. ", i + 1)),
                Span::styled(format!("Thread {:02}", thread_id), Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::styled(format!("{} shares", accepted), Style::default().fg(Color::Green)),
                Span::raw(format!(" ({:.1}% of total)", percentage)),
            ]));
        }
        
        // Problem analysis
        leader_lines.push(Line::from(""));
        leader_lines.push(Line::from(vec![
            Span::styled("Problem Analysis", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        leader_lines.push(Line::from(""));
        
        let mut has_problems = false;
        for (thread_id, accepted, rejected) in &thread_shares {
            if *rejected > 0 {
                let total = accepted + rejected;
                let reject_rate = (*rejected as f64 / total as f64) * 100.0;
                if reject_rate > 5.0 {
                    has_problems = true;
                    leader_lines.push(Line::from(vec![
                        Span::styled(format!("Thread {:02}", thread_id), Style::default().fg(Color::Red)),
                        Span::raw(": "),
                        Span::raw(format!("{}/{} ({:.1}%)", rejected, total, reject_rate)),
                        Span::styled(" ⚠️", Style::default().fg(Color::Yellow)),
                    ]));
                }
            }
        }
        
        if !has_problems {
            leader_lines.push(Line::from(vec![
                Span::styled("All threads operating normally", Style::default().fg(Color::Green)),
            ]));
        }
        
        let leaders_block = Paragraph::new(leader_lines)
            .block(Block::default().borders(Borders::ALL).title("Analysis"));
        f.render_widget(leaders_block, shares_chunks[1]);
    }

    fn draw_share_summary(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        
        // Left side - Share summary stats
        let total_shares = self.stats.shares_submitted.load(Ordering::Relaxed);
        let accepted = self.stats.shares_accepted.load(Ordering::Relaxed);
        let rejected = self.stats.shares_rejected.load(Ordering::Relaxed);
        
        let acceptance_rate = if total_shares > 0 {
            (accepted as f64 / total_shares as f64) * 100.0
        } else {
            0.0
        };
        
        let best_share = self.stats.thread_stats.iter()
            .map(|t| t.best_difficulty.load(Ordering::Relaxed))
            .max()
            .unwrap_or(0);
        
        let avg_luck = if let Ok(shares) = self.stats.recent_shares.lock() {
            if !shares.is_empty() {
                let total_luck: f64 = shares.iter()
                    .map(|s| s.difficulty as f64 / s.target as f64)
                    .sum();
                total_luck / shares.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        let last_share_time = self.stats.recent_shares.lock().unwrap()
            .back()
            .map(|s| s.time.elapsed())
            .unwrap_or(Duration::from_secs(999));
        
        let summary_lines = vec![
            Line::from(vec![
                Span::styled("Total: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} shares", total_shares), Style::default().fg(Color::Cyan)),
                Span::raw("    "),
                Span::styled("Accepted: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ({:.1}%)", accepted, acceptance_rate), Style::default().fg(Color::Green)),
                Span::raw("    "),
                Span::styled("Rejected: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ({:.1}%)", rejected, 100.0 - acceptance_rate), Style::default().fg(if rejected == 0 { Color::Green } else { Color::Red })),
            ]),
            Line::from(vec![
                Span::styled("Best Share: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_number(best_share), Style::default().fg(Color::Magenta)),
                Span::raw("    "),
                Span::styled("Avg Luck: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:.1}x", avg_luck), Style::default().fg(Color::Blue)),
                Span::raw("    "),
                Span::styled("Last Share: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(MinerStats::format_duration(last_share_time), Style::default().fg(Color::Cyan)),
            ]),
        ];
        
        let summary_block = Paragraph::new(summary_lines)
            .block(Block::default().borders(Borders::ALL).title("Share Summary"));
        f.render_widget(summary_block, chunks[0]);
        
        // Right side - Share rate graph
        let graph_block = Block::default()
            .borders(Borders::ALL)
            .title("Share Rate Trend (shares/minute)");
        f.render_widget(graph_block, chunks[1]);
        
        let rate_graph = self.create_share_rate_graph();
        let graph = Paragraph::new(rate_graph);
        f.render_widget(graph, chunks[1]);
    }

    fn create_hashrate_graph(&self) -> Vec<Line<'static>> {
        let history = self.stats.hashrate_history.lock().unwrap();
        if history.is_empty() {
            return vec![Line::from("No data yet...")];
        }
        
        let max_points = 20;
        let height = 4;
        
        // Calculate hashrates at intervals
        let mut rates = Vec::new();
        let now = Instant::now();
        let interval = Duration::from_secs(15);
        
        for i in 0..max_points {
            let target_time = now - (interval * (max_points - i - 1) as u32);
            
            // Find closest data point
            let mut closest_hashes = 0u64;
            for (time, hashes) in history.iter() {
                if *time <= target_time {
                    closest_hashes = *hashes;
                }
            }
            
            let rate = if closest_hashes > 0 {
                let elapsed = self.stats.start_time.elapsed().as_secs_f64();
                closest_hashes as f64 / elapsed
            } else {
                0.0
            };
            
            rates.push(rate);
        }
        
        let max_rate = rates.iter().fold(0.0f64, |a, &b| a.max(b));
        if max_rate == 0.0 {
            return vec![Line::from("Waiting for data...")];
        }
        
        let mut lines = vec![];
        
        // Create graph
        for h in (0..height).rev() {
            let threshold = max_rate * (h as f64 + 1.0) / height as f64;
            let mut line_str = String::new();
            
            for rate in &rates {
                if *rate >= threshold {
                    line_str.push('█');
                } else if *rate >= threshold - (max_rate / height as f64 / 2.0) {
                    line_str.push('▄');
                } else {
                    line_str.push(' ');
                }
            }
            
            lines.push(Line::from(line_str));
        }
        
        // Add scale
        lines.push(Line::from(format!("└{} {}", "─".repeat(20), MinerStats::format_hashrate(max_rate))));
        lines.push(Line::from("  5m  4m  3m  2m  1m  now"));
        
        lines
    }

    fn create_share_timeline(&self) -> Vec<Line<'static>> {
        let shares = self.stats.recent_shares.lock().unwrap();
        if shares.is_empty() {
            return vec![Line::from("No shares found yet...")];
        }
        
        let mut lines = vec![];
        let now = Instant::now();
        
        // Get last 10 shares
        let recent: Vec<_> = shares.iter().rev().take(10).collect();
        
        for share in recent.iter() {
            let time_ago = now.duration_since(share.time);
            let mins_ago = time_ago.as_secs() / 60;
            let secs_ago = time_ago.as_secs() % 60;
            
            let time_str = if mins_ago > 0 {
                format!("{:02}:{:02}", mins_ago, secs_ago)
            } else {
                format!("   :{:02}", secs_ago)
            };
            
            let symbol = if share.accepted { "●" } else { "○" };
            
            lines.push(Line::from(format!("{} {} Thread {:02} ({}x luck)",
                time_str,
                symbol,
                share.thread_id,
                (share.difficulty as f64 / share.target as f64) as u64
            )));
        }
        
        lines.push(Line::from(""));
        lines.push(Line::from("(● = accepted, ○ = rejected)"));
        
        lines
    }

    fn create_share_rate_graph(&self) -> Vec<Line<'static>> {
        let shares = self.stats.recent_shares.lock().unwrap();
        if shares.len() < 2 {
            return vec![Line::from("Insufficient data...")];
        }
        
        // Calculate share rates per minute over time
        let mut rates = Vec::new();
        let now = Instant::now();
        let interval = Duration::from_secs(60);
        
        for i in 0..10 {
            let window_start = now - (interval * (10 - i));
            let window_end = window_start + interval;
            
            let count = shares.iter()
                .filter(|s| s.time >= window_start && s.time < window_end)
                .count();
            
            rates.push(count as f64);
        }
        
        let max_rate = rates.iter().fold(0.0f64, |a, &b| a.max(b));
        if max_rate == 0.0 {
            return vec![Line::from("No shares in recent history...")];
        }
        
        let height = 4;
        let mut lines = vec![];
        
        // Create graph
        for h in (0..height).rev() {
            let threshold = max_rate * (h as f64 + 1.0) / height as f64;
            let mut line_str = String::new();
            
            line_str.push_str(&format!("{:2} ", (threshold as u64)));
            
            for rate in &rates {
                if *rate >= threshold {
                    line_str.push_str("██");
                } else if *rate >= threshold - (max_rate / height as f64 / 2.0) {
                    line_str.push_str("▄▄");
                } else {
                    line_str.push_str("  ");
                }
            }
            
            lines.push(Line::from(line_str));
        }
        
        lines.push(Line::from("   └10m─8m─6m─4m─2m─now"));
        
        lines
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stream = self.connect_to_pool().await?;
        info!("✅ Connected to pool");
        self.stats.add_activity("🔐 Connected to pool".to_string());
        
        let (reader, writer) = stream.into_split();
        let writer = Arc::new(tokio::sync::Mutex::new(writer));
        
        // Login
        self.login(&mut *writer.lock().await).await?;
        info!("🔐 Login request sent");
        
        // Create channels
        let (job_tx, _) = broadcast::channel(16);
        let (share_tx, share_rx) = mpsc::unbounded_channel();
        
        // Start threads
        self.start_mining_threads(job_tx.clone(), share_tx);
        self.start_share_submitter(Arc::clone(&writer), share_rx);
        self.start_stats_printer();
        
        // Start TUI if enabled
        let tui_handle = if self.enable_tui {
            let stats = Arc::clone(&self.stats);
            let pool_address = self.pool_address;
            let worker_name = self.worker_name.clone();
            let wallet_address = self.wallet_address.clone();
            let num_threads = self.num_threads;
            
            Some(tokio::spawn(async move {
                let miner = SHA3xMiner {
                    wallet_address,
                    pool_address,
                    worker_name,
                    num_threads,
                    stats,
                    enable_tui: true,
                };
                
                if let Err(e) = miner.run_tui().await {
                    error!("TUI error: {}", e);
                }
            }))
        } else {
            None
        };
        
        // Read pool messages
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        
        while let Some(line) = lines.next_line().await? {
            if let Err(e) = self.handle_pool_message(&line, &job_tx).await {
                error!("Error handling pool message: {}", e);
            }
        }
        
        if let Some(handle) = tui_handle {
            handle.abort();
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    if !args.tui {
        tracing_subscriber::fmt::init();
    }
    
    info!("🚀 Starting SHA3x Tari Miner");
    info!("📍 Pool: {}", args.pool);
    info!("💳 Wallet: {}", args.wallet);
    info!("👷 Worker: {}", args.worker);
    info!("🧵 Threads: {}", if args.threads == 0 { "auto".to_string() } else { args.threads.to_string() });
    
    let pool_address: SocketAddr = args.pool.parse()?;
    
    let miner = SHA3xMiner::new(
        args.wallet,
        pool_address,
        args.worker,
        args.threads,
        args.tui,
    );
    
    miner.run().await?;
    
    Ok(())
}

// SHA3x Tari Miner v3.0 - Complete TUI Edition
// Features:
// - Triple SHA3-256 hashing for Tari's SHA3x algorithm
// - Multi-threaded CPU mining with configurable thread count
// - Real-time TUI dashboard with 3 tabs (Overview, Threads, Shares)
// - Individual thread performance tracking
// - Dynamic 24x3 grid showing only active threads
// - VarDiff support with automatic difficulty adjustment
// - Comprehensive statistics and performance history
// - Share luck analysis with visual indicators
// - ASCII graphs for hashrate and share rate trends
// - Color-coded status indicators (green for good, red for errors only)
// - Recent activity feed with pool messages
// - Thread performance leaders and problem analysis