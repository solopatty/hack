use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
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

struct TEEOrderMatcher {
    order_book: HashMap<String, Vec<TradeIntent>>,
    batch_interval: u64,
    last_batch_time: u64,
    new_balance_state: HashMap<String, HashMap<String, i64>>, // Track token balances per trader
}
c
impl TEEOrderMatcher {
    fn new(batch_interval: u64) -> Self {
        Self {
            order_book: HashMap::new(),
            batch_interval,
            last_batch_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            new_balance_state: HashMap::new(),
        }
    }

    fn add_trade_intent(&mut self, intent: TradeIntent) {
        if !intent.is_expired() {
            // For debugging
            println!("Adding trade intent: {} wants to sell {} {} for {} {}", 
                intent.trader, intent.sell_amount, intent.sell_token, intent.buy_amount, intent.buy_token);
            self.order_book.entry(intent.sell_token.clone()).or_default().push(intent);
        }
    }

    fn match_orders(&mut self) -> Vec<(TradeIntent, TradeIntent)> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        // Force the matching to happen by removing the time check
        // if now - self.last_batch_time < self.batch_interval {
        //     return vec![];
        // }
        self.last_batch_time = now;

        let mut matches = Vec::new();
        let mut matched_traders = Vec::new();

        // Debug the order book
        println!("Order book state:");
        for (token, intents) in &self.order_book {
            println!("Token {}: {} orders", token, intents.len());
            for intent in intents {
                println!("  {} wants to sell {} {} for {} {} (price limit: {})", 
                    intent.trader, intent.sell_amount, intent.sell_token, 
                    intent.buy_amount, intent.buy_token, intent.price_limit);
            }
        }

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
                        
                        println!("Checking match between {} and {}", intent.trader, counter_intent.trader);
                        println!("  {} offers {} {} for {} {} (min price: {})", 
                            intent.trader, intent.sell_amount, intent.sell_token, 
                            intent.buy_amount, intent.buy_token, intent.price_limit);
                        println!("  {} offers {} {} for {} {} (min price: {})", 
                            counter_intent.trader, counter_intent.sell_amount, counter_intent.sell_token, 
                            counter_intent.buy_amount, counter_intent.buy_token, counter_intent.price_limit);
                        println!("  Intent price: {}, Counter price: {}", intent_price, counter_price);
                        
                        // Check if it's a valid match - simplified logic
                        if counter_intent.buy_token == *sell_token && 
                           counter_intent.sell_token == intent.buy_token {
                            
                            println!("  Tokens match!");
                            
                            // Check amounts
                            if intent.sell_amount == counter_intent.buy_amount &&
                               intent.buy_amount == counter_intent.sell_amount {
                                
                                println!("  Amounts match!");
                                
                                // Check price limits - this is where we likely had an issue
                                if intent_price >= intent.price_limit && 
                                   1.0/counter_price <= counter_intent.price_limit {
                                    
                                    println!("  Price limits satisfied! Match found!");
                                    
                                    matches.push((intent.clone(), counter_intent.clone()));
                                    matched_traders.push(intent.trader.clone());
                                    matched_traders.push(counter_intent.trader.clone());
                                    self.update_balance_state(intent, counter_intent);
                                    break; // Move to next intent after finding a match
                                } else {
                                    println!("  Price limits not satisfied");
                                }
                            } else {
                                println!("  Amounts don't match");
                            }
                        } else {
                            println!("  Tokens don't match");
                        }
                    }
                }
            }
        }
        
        // Clean up the order book
        self.clean_order_book(&matched_traders);
        
        matches
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
        format!("TEE Attestation: {:?}", self.new_balance_state)
    }
}

fn main() {
    let mut matcher = TEEOrderMatcher::new(5);
    
    // Alice wants to sell 1 ETH for 2000 DAI
    matcher.add_trade_intent(TradeIntent {
        trader: "Alice".to_string(),
        sell_token: "ETH".to_string(),
        buy_token: "DAI".to_string(),
        sell_amount: 1,
        buy_amount: 2000,
        price_limit: 2000.0,  // Alice wants at least 2000 DAI per ETH
        expiry: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
    });

    // Bob wants to sell 2000 DAI for 1 ETH
    matcher.add_trade_intent(TradeIntent {
        trader: "Bob".to_string(),
        sell_token: "DAI".to_string(),
        buy_token: "ETH".to_string(),
        sell_amount: 2000,
        buy_amount: 1,
        price_limit: 0.0005,  // Bob is willing to pay at most 2000 DAI per ETH (1/2000 = 0.0005)
        expiry: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
    });

    let matched_orders = matcher.match_orders();
    println!("Matched Trades: {:?}", matched_orders);
    println!("New Balance State: {:?}", matcher.new_balance_state);
    println!("Attestation: {}", matcher.generate_attestation());
}