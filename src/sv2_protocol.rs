// SHA3x Miner - Free and Open Source Software Statement
//
// This project, sha3x-miner, is Free and Open Source Software (FOSS) licensed
// under the MIT License. You are free to use, modify, and distribute this
// software in accordance with the license terms. Contributions are welcome
// via pull requests to the project repository.
//
// File: src/pool/sv2_protocol.rs
// Version: 2.0.0
// Developer: OIEIEIO <oieieio@protonmail.com>
//
// This file implements the Stratum V2 protocol for communication with Bitcoin
// mining pools in the SHA3x miner. It handles SV2 message construction,
// encoding, and decoding for standard channel mining.
//
// Tree Location:
// - src/pool/sv2_protocol.rs (SV2 protocol logic)
// - Depends on: binary_sv2, codec_sv2, roles_logic_sv2

use binary_sv2::{
    U256 as SV2U256,
    SetupConnection, SetupConnectionSuccess, SetupConnectionError,
    OpenStandardMiningChannel, OpenStandardMiningChannelSuccess, OpenMiningChannelError,
    NewMiningJob, SubmitSharesStandard, SubmitSharesSuccess, SubmitSharesError,
    SetTarget, CloseChannel, UpdateChannel,
};
use codec_sv2::{StandardEitherFrame, StandardSv2Frame, Encoder, Decoder};
use roles_logic_sv2::mining_sv2::{MiningDeviceMessages, PoolMessages};
use crate::core::types::{Algorithm, Sv2Channel, Sv2Job};
use crate::core::difficulty::U256;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{debug, error, info, warn};

/// SV2 Protocol handler for Bitcoin mining
pub struct Sv2Protocol {
    /// Current sequence number for requests
    request_id_counter: AtomicU32,
    /// Current sequence number for share submissions
    sequence_counter: AtomicU32,
    /// Encoder for outgoing messages
    encoder: Encoder<MiningDeviceMessages>,
    /// Decoder for incoming messages
    decoder: Decoder<PoolMessages>,
}

impl Sv2Protocol {
    /// Create a new SV2 protocol handler
    pub fn new() -> Self {
        Self {
            request_id_counter: AtomicU32::new(1),
            sequence_counter: AtomicU32::new(1),
            encoder: Encoder::new(),
            decoder: Decoder::new(),
        }
    }

    /// Get the next request ID
    fn next_request_id(&self) -> u32 {
        self.request_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Get the next sequence number
    fn next_sequence(&self) -> u32 {
        self.sequence_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Create a SetupConnection message for Bitcoin mining
    pub fn create_setup_connection(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let setup_msg = SetupConnection {
            protocol: 0, // Mining protocol
            min_version: 2,
            max_version: 2,
            flags: 0b001, // REQUIRES_STANDARD_JOBS flag
            endpoint_host: "0.0.0.0".to_string().try_into()?,
            endpoint_port: 0,
            vendor: "sha3x-miner".to_string().try_into()?,
            hardware_version: "2.0.0".to_string().try_into()?,
            firmware: "sha3x-miner-sv2".to_string().try_into()?,
            device_id: "sha3x-cpu-miner".to_string().try_into()?,
        };

        let message = MiningDeviceMessages::SetupConnection(setup_msg);
        let frame = StandardSv2Frame::from_message(message, 0, false)?;
        let encoded = self.encoder.encode(StandardEitherFrame::Standard(frame))?;
        
        debug!("Created SetupConnection message: {} bytes", encoded.len());
        Ok(encoded)
    }

    /// Create an OpenStandardMiningChannel message
    pub fn create_open_channel(
        &self,
        user_identity: &str,
        nominal_hashrate: f32,
        max_target: SV2U256,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let open_msg = OpenStandardMiningChannel {
            request_id: self.next_request_id(),
            user_identity: user_identity.to_string().try_into()?,
            nominal_hash_rate: nominal_hashrate,
            max_target,
        };

        let message = MiningDeviceMessages::OpenStandardMiningChannel(open_msg);
        let frame = StandardSv2Frame::from_message(message, 0, false)?;
        let encoded = self.encoder.encode(StandardEitherFrame::Standard(frame))?;
        
        info!("Created OpenStandardMiningChannel for user: {}, hashrate: {} H/s", 
              user_identity, nominal_hashrate);
        Ok(encoded)
    }

    /// Create a SubmitSharesStandard message
    pub fn create_submit_shares(
        &self,
        channel_id: u32,
        job_id: u32,
        nonce: u32,
        ntime: u32,
        version: u32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let submit_msg = SubmitSharesStandard {
            channel_id,
            sequence_number: self.next_sequence(),
            job_id,
            nonce,
            ntime,
            version,
        };

        let message = MiningDeviceMessages::SubmitSharesStandard(submit_msg);
        let frame = StandardSv2Frame::from_message(message, 0, false)?;
        let encoded = self.encoder.encode(StandardEitherFrame::Standard(frame))?;
        
        debug!("Created SubmitSharesStandard: channel={}, job={}, nonce={:08x}, ntime={:08x}", 
               channel_id, job_id, nonce, ntime);
        Ok(encoded)
    }

    /// Create an UpdateChannel message to report hashrate changes
    pub fn create_update_channel(
        &self,
        channel_id: u32,
        nominal_hashrate: f32,
        max_target: SV2U256,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let update_msg = UpdateChannel {
            channel_id,
            nominal_hash_rate: nominal_hashrate,
            maximum_target: max_target,
        };

        let message = MiningDeviceMessages::UpdateChannel(update_msg);
        let frame = StandardSv2Frame::from_message(message, 0, false)?;
        let encoded = self.encoder.encode(StandardEitherFrame::Standard(frame))?;
        
        debug!("Created UpdateChannel: channel={}, hashrate={} H/s", channel_id, nominal_hashrate);
        Ok(encoded)
    }

    /// Create a CloseChannel message
    pub fn create_close_channel(
        &self,
        channel_id: u32,
        reason: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let close_msg = CloseChannel {
            channel_id,
            reason_code: reason.to_string().try_into()?,
        };

        let message = MiningDeviceMessages::CloseChannel(close_msg);
        let frame = StandardSv2Frame::from_message(message, 0, false)?;
        let encoded = self.encoder.encode(StandardEitherFrame::Standard(frame))?;
        
        info!("Created CloseChannel: channel={}, reason={}", channel_id, reason);
        Ok(encoded)
    }

    /// Decode incoming SV2 messages from the pool
    pub fn decode_message(&mut self, data: &[u8]) -> Result<Vec<Sv2Message>, Box<dyn std::error::Error>> {
        let mut messages = Vec::new();
        let mut remaining_data = data;

        while !remaining_data.is_empty() {
            match self.decoder.next_frame(remaining_data) {
                Ok((consumed, Some(frame))) => {
                    remaining_data = &remaining_data[consumed..];
                    
                    match frame {
                        StandardEitherFrame::Standard(sv2_frame) => {
                            if let Ok(message) = sv2_frame.payload() {
                                let sv2_msg = self.parse_pool_message(message)?;
                                messages.push(sv2_msg);
                            }
                        }
                        _ => {
                            warn!("Received non-standard SV2 frame, ignoring");
                        }
                    }
                }
                Ok((consumed, None)) => {
                    // Need more data
                    remaining_data = &remaining_data[consumed..];
                    break;
                }
                Err(e) => {
                    error!("Failed to decode SV2 frame: {}", e);
                    break;
                }
            }
        }

        Ok(messages)
    }

    /// Parse a pool message into our internal representation
    fn parse_pool_message(&self, message: PoolMessages) -> Result<Sv2Message, Box<dyn std::error::Error>> {
        match message {
            PoolMessages::SetupConnectionSuccess(msg) => {
                info!("✅ SV2 Setup successful - flags: {:032b}", msg.flags);
                Ok(Sv2Message::SetupSuccess {
                    flags: msg.flags,
                })
            }
            PoolMessages::SetupConnectionError(msg) => {
                error!("❌ SV2 Setup failed: {}", std::str::from_utf8(&msg.error_code)?);
                Ok(Sv2Message::SetupError {
                    error_code: msg.error_code.to_vec(),
                })
            }
            PoolMessages::OpenStandardMiningChannelSuccess(msg) => {
                info!("✅ SV2 Channel opened - ID: {}, target: {:064x}", 
                      msg.channel_id, msg.target);
                Ok(Sv2Message::ChannelSuccess {
                    request_id: msg.request_id,
                    channel_id: msg.channel_id,
                    target: msg.target,
                    extranonce_prefix: msg.extranonce_prefix.to_vec(),
                    group_channel_id: msg.group_channel_id,
                })
            }
            PoolMessages::OpenMiningChannelError(msg) => {
                error!("❌ SV2 Channel failed: {}", std::str::from_utf8(&msg.error_code)?);
                Ok(Sv2Message::ChannelError {
                    request_id: msg.request_id,
                    error_code: msg.error_code.to_vec(),
                })
            }
            PoolMessages::NewMiningJob(msg) => {
                info!("📋 SV2 New job - Channel: {}, Job: {}, Version: {:08x}", 
                      msg.channel_id, msg.job_id, msg.version);
                Ok(Sv2Message::NewJob {
                    channel_id: msg.channel_id,
                    job_id: msg.job_id,
                    min_ntime: msg.min_ntime,
                    version: msg.version,
                    merkle_root: msg.merkle_root,
                })
            }
            PoolMessages::SetTarget(msg) => {
                info!("🔧 SV2 Target update - Channel: {}, Target: {:064x}", 
                      msg.channel_id, msg.maximum_target);
                Ok(Sv2Message::SetTarget {
                    channel_id: msg.channel_id,
                    target: msg.maximum_target,
                })
            }
            PoolMessages::SubmitSharesSuccess(msg) => {
                info!("✅ SV2 Share accepted - Channel: {}, Sequence: {}, Count: {}", 
                      msg.channel_id, msg.last_sequence_number, msg.new_submits_accepted_count);
                Ok(Sv2Message::ShareSuccess {
                    channel_id: msg.channel_id,
                    last_sequence: msg.last_sequence_number,
                    accepted_count: msg.new_submits_accepted_count,
                    shares_sum: msg.new_shares_sum,
                })
            }
            PoolMessages::SubmitSharesError(msg) => {
                error!("❌ SV2 Share rejected - Channel: {}, Sequence: {}, Error: {}", 
                       msg.channel_id, msg.sequence_number, std::str::from_utf8(&msg.error_code)?);
                Ok(Sv2Message::ShareError {
                    channel_id: msg.channel_id,
                    sequence_number: msg.sequence_number,
                    error_code: msg.error_code.to_vec(),
                })
            }
            _ => {
                warn!("Received unhandled SV2 message type");
                Ok(Sv2Message::Unknown)
            }
        }
    }

    /// Convert our internal U256 to SV2 U256
    pub fn convert_target_to_sv2(target: &U256) -> SV2U256 {
        let mut bytes = [0u8; 32];
        target.to_big_endian(&mut bytes);
        SV2U256::from_big_endian(&bytes)
    }

    /// Convert SV2 U256 to our internal U256
    pub fn convert_target_from_sv2(sv2_target: &SV2U256) -> U256 {
        let bytes = sv2_target.to_big_endian();
        U256::from_big_endian(&bytes)
    }

    /// Calculate maximum target for given hashrate (helper function)
    pub fn calculate_max_target(hashrate: f32) -> SV2U256 {
        // Set a reasonable max target based on hashrate
        // Higher hashrate = can handle higher difficulty (lower target)
        let base_target = if hashrate > 1_000_000.0 {
            // > 1 MH/s - can handle higher difficulty
            SV2U256::from_big_endian(&[
                0x00, 0x00, 0x00, 0x00, 0x0F, 0xFF, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        } else if hashrate > 100_000.0 {
            // 100 KH/s - 1 MH/s
            SV2U256::from_big_endian(&[
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        } else {
            // < 100 KH/s - very easy target
            SV2U256::from_big_endian(&[
                0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        };

        debug!("Calculated max target for hashrate {} H/s: {:064x}", hashrate, base_target);
        base_target
    }
}

/// Internal representation of SV2 messages
#[derive(Debug, Clone)]
pub enum Sv2Message {
    SetupSuccess {
        flags: u32,
    },
    SetupError {
        error_code: Vec<u8>,
    },
    ChannelSuccess {
        request_id: u32,
        channel_id: u32,
        target: SV2U256,
        extranonce_prefix: Vec<u8>,
        group_channel_id: u32,
    },
    ChannelError {
        request_id: u32,
        error_code: Vec<u8>,
    },
    NewJob {
        channel_id: u32,
        job_id: u32,
        min_ntime: Option<u32>,
        version: u32,
        merkle_root: SV2U256,
    },
    SetTarget {
        channel_id: u32,
        target: SV2U256,
    },
    ShareSuccess {
        channel_id: u32,
        last_sequence: u32,
        accepted_count: u32,
        shares_sum: u64,
    },
    ShareError {
        channel_id: u32,
        sequence_number: u32,
        error_code: Vec<u8>,
    },
    Unknown,
}

impl Sv2Message {
    /// Convert SV2 job message to our internal Sv2Job type
    pub fn to_sv2_job(&self) -> Option<Sv2Job> {
        match self {
            Sv2Message::NewJob {
                channel_id,
                job_id,
                min_ntime,
                version,
                merkle_root,
            } => Some(Sv2Job::new(
                *channel_id,
                *job_id,
                *min_ntime,
                *version,
                *merkle_root,
            )),
            _ => None,
        }
    }

    /// Convert SV2 channel success to our internal Sv2Channel type
    pub fn to_sv2_channel(&self, nominal_hashrate: f32) -> Option<Sv2Channel> {
        match self {
            Sv2Message::ChannelSuccess {
                channel_id,
                target,
                ..
            } => {
                let max_target = Sv2Protocol::calculate_max_target(nominal_hashrate);
                Some(Sv2Channel::new(*channel_id, *target, nominal_hashrate, max_target))
            }
            _ => None,
        }
    }

    /// Check if this is an error message
    pub fn is_error(&self) -> bool {
        matches!(self, Sv2Message::SetupError { .. } | 
                      Sv2Message::ChannelError { .. } | 
                      Sv2Message::ShareError { .. })
    }

    /// Get error description if this is an error message
    pub fn error_description(&self) -> Option<String> {
        match self {
            Sv2Message::SetupError { error_code } |
            Sv2Message::ChannelError { error_code, .. } |
            Sv2Message::ShareError { error_code, .. } => {
                String::from_utf8(error_code.clone()).ok()
            }
            _ => None,
        }
    }
}

/// Helper functions for SV2 protocol constants
impl Sv2Protocol {
    /// Get the protocol version we support
    pub const PROTOCOL_VERSION: u16 = 2;
    
    /// Get our vendor string
    pub const VENDOR: &'static str = "sha3x-miner";
    
    /// Get our hardware version
    pub const HARDWARE_VERSION: &'static str = "2.0.0";
    
    /// Get our firmware version
    pub const FIRMWARE_VERSION: &'static str = "sha3x-miner-sv2";
    
    /// Get our device ID
    pub const DEVICE_ID: &'static str = "sha3x-cpu-miner";
    
    /// Standard mining protocol ID
    pub const MINING_PROTOCOL: u8 = 0;
    
    /// Flags for setup connection
    pub const REQUIRES_STANDARD_JOBS: u32 = 0b001;
    pub const REQUIRES_WORK_SELECTION: u32 = 0b010;
    pub const REQUIRES_VERSION_ROLLING: u32 = 0b100;
}

// Changelog:
// - v2.0.0 (2025-06-19): Complete SV2 protocol implementation.
//   - Replaces old JSON-RPC Stratum V1 protocol with binary SV2.
//   - Implements all core SV2 messages: SetupConnection, OpenStandardMiningChannel, SubmitSharesStandard.
//   - Provides message encoding/decoding with proper error handling.
//   - Includes helper functions for target conversion and channel management.
//   - Supports standard channel mining for Bitcoin SHA-256d algorithm.
//   - Compatible with binary_sv2, codec_sv2, and roles_logic_sv2 crates.
//   - Clean separation from SHA3x protocol (which remains JSON-based).