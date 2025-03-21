use std::{net::ToSocketAddrs, str::from_utf8, collections::HashMap, time::{SystemTime, UNIX_EPOCH, Duration}};

use actix_web::web::BytesMut;
use actix_web::{
    dev::PeerAddr, error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::Context;
use awc::Client;
use clap::Parser;
use ethabi::Token;
use k256::elliptic_curve::generic_array::sequence::Lengthen;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use tokio::fs;
use tokio_stream::StreamExt;
use url::Url;

// Trade Intent and Matching Engine Structures

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TradeIntent {
    trader: String,
    sell_token: String,
    buy_token: String,
    sell_amount: u64,
    buy_amount: u64,
    price_limit: f64,
    expiry: u64,
}

impl TradeIntent {
    fn is_expired(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now > self.expiry
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedTradeIntent {
    encrypted_data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BalanceUpdate {
    trader_balances: HashMap<String, HashMap<String, i64>>,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeRequest {
    encrypted_intents: Vec<EncryptedTradeIntent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeResponse {
    attestation: String,
    matches: Vec<MatchedTrade>,
    balance_state: HashMap<String, HashMap<String, i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchedTrade {
    trader1: String,
    trader2: String,
    token1: String,
    token2: String,
    amount1: u64,
    amount2: u64,
}

struct TEEOrderMatcher {
    order_book: HashMap<String, Vec<TradeIntent>>,
    batch_interval: u64,
    last_batch_time: u64,
    new_balance_state: HashMap<String, HashMap<String, i64>>,
    tee_private_key: Vec<u8>,
    tee_signing_key: Vec<u8>,
}

impl TEEOrderMatcher {
    fn new(batch_interval: u64, tee_private_key: Vec<u8>, tee_signing_key: Vec<u8>) -> Self {
        Self {
            order_book: HashMap::new(),
            batch_interval,
            last_batch_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            new_balance_state: HashMap::new(),
            tee_private_key,
            tee_signing_key,
        }
    }

    fn decrypt_trade_intent(&self, encrypted_intent: &EncryptedTradeIntent) -> Result<TradeIntent, String> {
        // In a real implementation, this would decrypt using the TEE's private key
        // For this example, we'll simulate decryption with a simple deserialization
        
        // Mock implementation - in production this would use proper decryption
        if encrypted_intent.encrypted_data.is_empty() {
            return Err("Empty encrypted data".to_string());
        }
        
        // For demo purposes only - this is not actual decryption
        match serde_json::from_slice(&encrypted_intent.encrypted_data) {
            Ok(intent) => Ok(intent),
            Err(e) => Err(format!("Failed to decrypt: {}", e)),
        }
    }

    fn add_encrypted_trade_intent(&mut self, encrypted_intent: &EncryptedTradeIntent) -> Result<(), String> {
        let intent = self.decrypt_trade_intent(encrypted_intent)?;
        self.add_trade_intent(intent);
        Ok(())
    }

    fn add_trade_intent(&mut self, intent: TradeIntent) {
        if !intent.is_expired() {
            log::info!("Adding trade intent: {} wants to sell {} {} for {} {}", 
                intent.trader, intent.sell_amount, intent.sell_token, intent.buy_amount, intent.buy_token);
            self.order_book.entry(intent.sell_token.clone()).or_default().push(intent);
        }
    }

    fn match_orders(&mut self) -> Vec<MatchedTrade> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - self.last_batch_time < self.batch_interval {
            return vec![];
        }
        self.last_batch_time = now;

        let mut matched_trades = Vec::new();
        let mut matched_traders = Vec::new();

        // Create a copy of the order book to avoid borrow checker issues
        let order_book_copy = self.order_book.clone();

        for (sell_token, intents) in &order_book_copy {
            for intent in intents.iter() {
                if intent.is_expired() || matched_traders.contains(&intent.trader) {
                    continue;
                }
                
                if let Some(counter_intents) = order_book_copy.get(&intent.buy_token) {
                    for counter_intent in counter_intents.iter() {
                        if counter_intent.is_expired() || matched_traders.contains(&counter_intent.trader) 
                            || counter_intent.trader == intent.trader {  // Avoid self-trading
                            continue;
                        }

                        // Calculate prices correctly
                        let intent_price = intent.buy_amount as f64 / intent.sell_amount as f64;
                        let counter_price = counter_intent.buy_amount as f64 / counter_intent.sell_amount as f64;
                        
                        // Check if it's a valid match
                        if counter_intent.buy_token == *sell_token && 
                           counter_intent.sell_token == intent.buy_token {
                            
                            // Check amounts
                            if intent.sell_amount == counter_intent.buy_amount &&
                               intent.buy_amount == counter_intent.sell_amount {
                                
                                // Check price limits
                                if intent_price >= intent.price_limit && 
                                   1.0/counter_price <= counter_intent.price_limit {
                                    
                                    log::info!("Match found between {} and {}", intent.trader, counter_intent.trader);
                                    
                                    matched_trades.push(MatchedTrade {
                                        trader1: intent.trader.clone(),
                                        trader2: counter_intent.trader.clone(),
                                        token1: intent.sell_token.clone(),
                                        token2: intent.buy_token.clone(),
                                        amount1: intent.sell_amount,
                                        amount2: intent.buy_amount,
                                    });
                                    
                                    matched_traders.push(intent.trader.clone());
                                    matched_traders.push(counter_intent.trader.clone());
                                    self.update_balance_state(intent, counter_intent);
                                    break; // Move to next intent after finding a match
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Clean up the order book
        self.clean_order_book(&matched_traders);
        
        matched_trades
    }
    
    fn clean_order_book(&mut self, matched_traders: &[String]) {
        // Create a list of tokens to process
        let tokens: Vec<String> = self.order_book.keys().cloned().collect();
        
        for token in tokens {
            // Remove expired and matched trades
            if let Some(intents) = self.order_book.get_mut(&token) {
                intents.retain(|intent| !intent.is_expired() && !matched_traders.contains(&intent.trader));
            }
        }
        
        // Remove any empty vectors
        self.order_book.retain(|_, intents| !intents.is_empty());
    }
    
    fn update_balance_state(&mut self, intent: &TradeIntent, counter_intent: &TradeIntent) {
        // Update first trader's balances
        let trader1_balances = self.new_balance_state
            .entry(intent.trader.clone())
            .or_insert_with(HashMap::new);
            
        *trader1_balances.entry(intent.sell_token.clone()).or_insert(0) -= intent.sell_amount as i64;
        *trader1_balances.entry(intent.buy_token.clone()).or_insert(0) += intent.buy_amount as i64;
        
        // Update second trader's balances
        let trader2_balances = self.new_balance_state
            .entry(counter_intent.trader.clone())
            .or_insert_with(HashMap::new);
            
        *trader2_balances.entry(counter_intent.sell_token.clone()).or_insert(0) -= counter_intent.sell_amount as i64;
        *trader2_balances.entry(counter_intent.buy_token.clone()).or_insert(0) += counter_intent.buy_amount as i64;
    }

    fn generate_attestation(&self) -> String {
        // Serialize the balance state
        let balance_update = BalanceUpdate {
            trader_balances: self.new_balance_state.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // In a real implementation, this would be signed
        format!("TEE Attestation: {}", serde_json::to_string(&balance_update).unwrap_or_default())
    }
}

// Global state for the TEE matcher
struct AppState {
    matcher: std::sync::Mutex<TEEOrderMatcher>,
}

// Handle trade intents
async fn process_trades(
    req: HttpRequest,
    mut payload: web::Payload,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let mut bytes = BytesMut::new();
    while let Some(item) = payload.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    // Parse the trade request
    let trade_request: TradeRequest = serde_json::from_slice(&bytes)
        .map_err(|e| error::ErrorBadRequest(format!("Invalid trade request: {}", e)))?;
    
    log::info!("Received {} encrypted trade intents", trade_request.encrypted_intents.len());
    
    // Get the TEE signer key for attestation
    let signer = k256::ecdsa::SigningKey::from_slice(
        fs::read("../app/secp.sec")
            .await
            .context("failed to read signer key")
            .unwrap()
            .as_slice(),
    )
    .context("invalid signer key")
    .unwrap();

    // Add the intents to the matcher
    {
        let mut matcher = app_state.matcher.lock().unwrap();
        for encrypted_intent in &trade_request.encrypted_intents {
            if let Err(e) = matcher.add_encrypted_trade_intent(encrypted_intent) {
                log::warn!("Failed to add encrypted trade intent: {}", e);
            }
        }
        
        // Match the orders
        let matched_trades = matcher.match_orders();
        log::info!("Matched {} trades", matched_trades.len());
        
        // Generate attestation for the balance updates
        let attestation = matcher.generate_attestation();
        
        // Create the response
        let response = TradeResponse {
            attestation,
            matches: matched_trades,
            balance_state: matcher.new_balance_state.clone(),
        };
        
        // Sign the response
        let mut hasher = Keccak::v256();
        hasher.update(b"|marlin-tee-matcher|");
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        hasher.update(b"|timestamp|");
        hasher.update(&timestamp.to_be_bytes());
        
        let response_bytes = serde_json::to_vec(&response).unwrap_or_default();
        hasher.update(b"|trade_response|");
        hasher.update(&response_bytes);
        
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);
        
        let (rs, v) = signer.sign_prehash_recoverable(&hash).unwrap();
        let signature = rs.to_bytes().append(27 + v.to_byte());
        
        // Create the HTTP response
        let mut client_resp = HttpResponse::Ok();
        client_resp.insert_header(("X-TEE-Timestamp", timestamp.to_string()));
        client_resp.insert_header(("X-TEE-Signature", hex::encode(signature.as_slice())));
        
        Ok(client_resp.json(response))
    }
}

// Health check endpoint
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("TEE Matcher Service Running")
}

#[derive(clap::Parser, Debug)]
struct CliArguments {
    listen_addr: String,
    listen_port: u16,
    batch_interval: Option<u64>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = CliArguments::parse();
    
    log::info!(
        "starting TEE matcher HTTP server at http://{}:{}",
        &args.listen_addr,
        args.listen_port
    );
    
    // In a real TEE environment, these would be securely stored or generated
    let tee_private_key = vec![1, 2, 3, 4]; // Placeholder
    let tee_signing_key = vec![5, 6, 7, 8]; // Placeholder
    
    // Create the matcher with the specified batch interval (default to 60 seconds)
    let batch_interval = args.batch_interval.unwrap_or(60);
    log::info!("Using batch interval of {} seconds", batch_interval);
    
    let matcher = TEEOrderMatcher::new(batch_interval, tee_private_key, tee_signing_key);
    
    // Create application state with the matcher
    let app_state = web::Data::new(AppState {
        matcher: std::sync::Mutex::new(matcher),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/process", web::post().to(process_trades))
    })
    .bind((args.listen_addr, args.listen_port))?
    .workers(2)
    .run()
    .await
}