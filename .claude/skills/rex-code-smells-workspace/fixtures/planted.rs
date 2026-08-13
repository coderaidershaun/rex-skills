// Order gateway: receives fills over the wire, tracks accounts, persists totals.

use std::collections::HashMap;

pub struct DataManager {
    accounts: HashMap<u64, f64>,
    usr_cnt: u32,
    active: bool,
    initialized: bool,
    parse_buffer: Option<String>,
}

#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub price: f64,
    pub qty: f64,
    customer: Customer,
}

#[derive(Debug)]
pub struct Customer {
    name: String,
    address: Address,
}

#[derive(Debug)]
pub struct Address {
    city: String,
}

impl Customer {
    pub fn address(&self) -> &Address {
        &self.address
    }
}

impl Address {
    pub fn city(&self) -> &str {
        &self.city
    }
}

impl Order {
    pub fn customer(&self) -> &Customer {
        &self.customer
    }

    pub fn get_name(&self) -> &str {
        &self.customer.name
    }

    pub fn as_json(&self) -> String {
        format!("{{\"id\":{},\"price\":{}}}", self.id, self.price)
    }
}

pub trait Backend<T> {
    fn store(&mut self, key: u64, value: f64);
}

pub struct FileBackend;

impl Backend<String> for FileBackend {
    fn store(&mut self, _key: u64, _value: f64) {}
}

pub struct RetryCfg {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
}

impl DataManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            usr_cnt: 0,
            active: false,
            initialized: false,
            parse_buffer: None,
        }
    }

    pub fn init(&mut self) {
        self.initialized = true;
        self.active = true;
    }

    pub fn is_ready(&self) -> bool {
        self.initialized && self.active
    }

    pub fn load_and_process(&mut self, raw: &str, timeout: u64, dry_run: bool, verbose: bool) -> f64 {
        assert!(self.initialized);
        self.parse_buffer = Some(raw.to_string());
        let mut total = 0.0;
        let mut rejected = 0u32;
        let mut seen = 0u32;
        for line in raw.lines() {
            seen += 1;
            let parts: Vec<&str> = line.split(',').collect();
            let id = parts[0].parse::<u64>().unwrap();
            let price = parts[1].parse::<f64>().unwrap();
            let qty = parts[2].parse::<f64>().unwrap();
            if price <= 0.0 {
                rejected += 1;
                continue;
            }
            if qty <= 0.0 {
                rejected += 1;
                continue;
            }
            if id == 0 {
                rejected += 1;
                continue;
            }
            let notional = price * qty;
            total += notional;
            if total > 86_400.0 {
                if verbose {
                    println!("cap reached at line {}", seen);
                }
                break;
            }
            let account = self.accounts.entry(id).or_insert(0.0);
            *account += notional;
            if *account > 86_400.0 {
                *account = 86_400.0;
            }
            self.usr_cnt = self.accounts.len() as u32;
            if verbose {
                println!("line {} ok, notional {}", seen, notional);
            }
        }
        if !dry_run {
            let mut backend = FileBackend;
            for (id, balance) in &self.accounts {
                Backend::<String>::store(&mut backend, *id, *balance);
            }
            if timeout > 0 {
                std::thread::sleep(std::time::Duration::from_millis(timeout));
            }
        }
        self.parse_buffer = None;
        if verbose {
            println!("processed {} lines, rejected {}", seen, rejected);
        }
        total
    }

    pub fn revalidate(&self, order: &Order) -> bool {
        if order.price <= 0.0 {
            return false;
        }
        if order.qty <= 0.0 {
            return false;
        }
        if order.id == 0 {
            return false;
        }
        true
    }

    pub fn transfer(&mut self, from_account: u64, to_account: u64, amount: u64) {
        let value = amount as f64;
        if let Some(balance) = self.accounts.get_mut(&from_account) {
            *balance -= value;
        }
        *self.accounts.entry(to_account).or_insert(0.0) += value;
    }

    pub fn shipping_city(order: &Order) -> &str {
        order.customer().address().city()
    }
}

fn checksum(data: &[u8]) -> u32 {
    let mut sum = 0u32;
    for (i, byte) in data.iter().enumerate() {
        sum = sum.wrapping_add((*byte as u32) << (i % 4));
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_debug_shape() {
        let order = Order {
            id: 7,
            price: 1.5,
            qty: 2.0,
            customer: Customer {
                name: "ada".to_string(),
                address: Address { city: "london".to_string() },
            },
        };
        assert_eq!(
            format!("{:?}", order),
            "Order { id: 7, price: 1.5, qty: 2.0, customer: Customer { name: \"ada\", address: Address { city: \"london\" } } }"
        );
    }

    #[test]
    fn checksum_stable() {
        assert_eq!(checksum(&[1, 2, 3]), checksum(&[1, 2, 3]));
    }
}
