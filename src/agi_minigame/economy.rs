use std::collections::HashMap;

use crate::base::value::{Value, ValueMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    Gold,
    Gem,
    Energy,
    Token,
    Custom(u32),
}

impl CurrencyType {
    pub fn name(&self) -> &str {
        match self {
            CurrencyType::Gold => "gold",
            CurrencyType::Gem => "gem",
            CurrencyType::Energy => "energy",
            CurrencyType::Token => "token",
            CurrencyType::Custom(id) => "custom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Currency {
    amounts: HashMap<CurrencyType, u64>,
    caps: HashMap<CurrencyType, u64>,
}

impl Currency {
    pub fn new() -> Self {
        let mut amounts = HashMap::new();
        let mut caps = HashMap::new();

        amounts.insert(CurrencyType::Gold, 0);
        amounts.insert(CurrencyType::Gem, 0);
        amounts.insert(CurrencyType::Energy, 100);
        caps.insert(CurrencyType::Gold, u64::MAX);
        caps.insert(CurrencyType::Gem, u64::MAX);
        caps.insert(CurrencyType::Energy, 200);

        Self { amounts, caps }
    }

    pub fn get(&self, currency_type: CurrencyType) -> u64 {
        self.amounts.get(&currency_type).copied().unwrap_or(0)
    }

    pub fn add(&mut self, currency_type: CurrencyType, amount: u64) -> u64 {
        let current = self.get(currency_type);
        let cap = self.caps.get(&currency_type).copied().unwrap_or(u64::MAX);
        let new_amount = current.saturating_add(amount).min(cap);
        self.amounts.insert(currency_type, new_amount);
        new_amount
    }

    pub fn spend(&mut self, currency_type: CurrencyType, amount: u64) -> bool {
        let current = self.get(currency_type);
        if current >= amount {
            self.amounts.insert(currency_type, current - amount);
            true
        } else {
            false
        }
    }

    pub fn set_cap(&mut self, currency_type: CurrencyType, cap: u64) {
        self.caps.insert(currency_type, cap);
    }

    pub fn get_cap(&self, currency_type: CurrencyType) -> u64 {
        self.caps.get(&currency_type).copied().unwrap_or(u64::MAX)
    }

    pub fn can_afford(&self, currency_type: CurrencyType, amount: u64) -> bool {
        self.get(currency_type) >= amount
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub entries: Vec<TransactionEntry>,
    pub timestamp: u64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TransactionEntry {
    pub currency_type: CurrencyType,
    pub amount: u64,
    pub is_gain: bool,
}

impl Transaction {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            entries: Vec::new(),
            timestamp: 0,
            description: description.to_string(),
        }
    }

    pub fn gain(mut self, currency_type: CurrencyType, amount: u64) -> Self {
        self.entries.push(TransactionEntry {
            currency_type,
            amount,
            is_gain: true,
        });
        self
    }

    pub fn cost(mut self, currency_type: CurrencyType, amount: u64) -> Self {
        self.entries.push(TransactionEntry {
            currency_type,
            amount,
            is_gain: false,
        });
        self
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn total_gains(&self, currency_type: CurrencyType) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.currency_type == currency_type && e.is_gain)
            .map(|e| e.amount)
            .sum()
    }

    pub fn total_costs(&self, currency_type: CurrencyType) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.currency_type == currency_type && !e.is_gain)
            .map(|e| e.amount)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub currency: Currency,
    pub transaction_log: Vec<Transaction>,
    pub max_log_size: usize,
}

impl Wallet {
    pub fn new() -> Self {
        Self {
            currency: Currency::new(),
            transaction_log: Vec::new(),
            max_log_size: 1000,
        }
    }

    pub fn execute(&mut self, transaction: Transaction) -> bool {
        for entry in &transaction.entries {
            if !entry.is_gain && !self.currency.can_afford(entry.currency_type, entry.amount) {
                return false;
            }
        }

        for entry in &transaction.entries {
            if entry.is_gain {
                self.currency.add(entry.currency_type, entry.amount);
            } else {
                self.currency.spend(entry.currency_type, entry.amount);
            }
        }

        if self.transaction_log.len() >= self.max_log_size {
            self.transaction_log.remove(0);
        }
        self.transaction_log.push(transaction);
        true
    }

    pub fn can_execute(&self, transaction: &Transaction) -> bool {
        for entry in &transaction.entries {
            if !entry.is_gain && !self.currency.can_afford(entry.currency_type, entry.amount) {
                return false;
            }
        }
        true
    }

    pub fn get_balance(&self, currency_type: CurrencyType) -> u64 {
        self.currency.get(currency_type)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub item_id: String,
    pub name: String,
    pub quantity: u32,
    pub max_stack: u32,
    pub metadata: ValueMap,
}

impl InventoryItem {
    pub fn new(item_id: &str, name: &str) -> Self {
        Self {
            item_id: item_id.to_string(),
            name: name.to_string(),
            quantity: 1,
            max_stack: 99,
            metadata: ValueMap::new(),
        }
    }

    pub fn with_quantity(mut self, qty: u32) -> Self {
        self.quantity = qty;
        self
    }

    pub fn with_max_stack(mut self, max: u32) -> Self {
        self.max_stack = max;
        self
    }

    pub fn add(&mut self, amount: u32) -> u32 {
        let new_qty = self.quantity.saturating_add(amount).min(self.max_stack);
        let added = new_qty - self.quantity;
        self.quantity = new_qty;
        added
    }

    pub fn remove(&mut self, amount: u32) -> u32 {
        let removed = amount.min(self.quantity);
        self.quantity -= removed;
        removed
    }

    pub fn is_full(&self) -> bool {
        self.quantity >= self.max_stack
    }

    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    items: HashMap<String, InventoryItem>,
    capacity: u32,
}

impl Inventory {
    pub fn new(capacity: u32) -> Self {
        Self {
            items: HashMap::new(),
            capacity,
        }
    }

    pub fn add_item(&mut self, item: InventoryItem) -> u32 {
        let item_id = item.item_id.clone();
        if let Some(existing) = self.items.get_mut(&item_id) {
            existing.add(item.quantity)
        } else {
            if (self.items.len() as u32) < self.capacity {
                let qty = item.quantity;
                self.items.insert(item_id, item);
                qty
            } else {
                0
            }
        }
    }

    pub fn remove_item(&mut self, item_id: &str, amount: u32) -> u32 {
        if let Some(item) = self.items.get_mut(item_id) {
            let removed = item.remove(amount);
            if item.is_empty() {
                self.items.remove(item_id);
            }
            removed
        } else {
            0
        }
    }

    pub fn get_item(&self, item_id: &str) -> Option<&InventoryItem> {
        self.items.get(item_id)
    }

    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut InventoryItem> {
        self.items.get_mut(item_id)
    }

    pub fn has_item(&self, item_id: &str, min_quantity: u32) -> bool {
        self.items
            .get(item_id)
            .map(|i| i.quantity >= min_quantity)
            .unwrap_or(false)
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() as u32 >= self.capacity
    }

    pub fn items(&self) -> impl Iterator<Item = &InventoryItem> {
        self.items.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_basic() {
        let mut c = Currency::new();
        assert_eq!(c.get(CurrencyType::Gold), 0);
        c.add(CurrencyType::Gold, 100);
        assert_eq!(c.get(CurrencyType::Gold), 100);
        assert!(c.spend(CurrencyType::Gold, 50));
        assert_eq!(c.get(CurrencyType::Gold), 50);
        assert!(!c.spend(CurrencyType::Gold, 100));
        assert_eq!(c.get(CurrencyType::Gold), 50);
    }

    #[test]
    fn test_currency_cap() {
        let mut c = Currency::new();
        c.set_cap(CurrencyType::Energy, 100);
        c.add(CurrencyType::Energy, 200);
        assert_eq!(c.get(CurrencyType::Energy), 100);
    }

    #[test]
    fn test_wallet_transaction() {
        let mut wallet = Wallet::new();
        wallet.currency.add(CurrencyType::Gold, 500);

        let tx = Transaction::new("tx1", "buy item")
            .cost(CurrencyType::Gold, 100)
            .gain(CurrencyType::Token, 1);

        assert!(wallet.execute(tx));
        assert_eq!(wallet.get_balance(CurrencyType::Gold), 400);
        assert_eq!(wallet.get_balance(CurrencyType::Token), 1);
    }

    #[test]
    fn test_wallet_insufficient_funds() {
        let mut wallet = Wallet::new();
        let tx = Transaction::new("tx2", "expensive")
            .cost(CurrencyType::Gold, 9999);
        assert!(!wallet.execute(tx));
    }

    #[test]
    fn test_inventory() {
        let mut inv = Inventory::new(10);
        let item = InventoryItem::new("sword", "Iron Sword").with_quantity(1);
        inv.add_item(item);
        assert!(inv.has_item("sword", 1));
        assert!(!inv.has_item("sword", 2));

        let item2 = InventoryItem::new("sword", "Iron Sword").with_quantity(3);
        inv.add_item(item2);
        assert!(inv.has_item("sword", 4));

        inv.remove_item("sword", 2);
        assert!(inv.has_item("sword", 2));
    }

    #[test]
    fn test_inventory_capacity() {
        let mut inv = Inventory::new(2);
        inv.add_item(InventoryItem::new("a", "A"));
        inv.add_item(InventoryItem::new("b", "B"));
        assert!(inv.is_full());
        inv.add_item(InventoryItem::new("c", "C"));
        assert_eq!(inv.count(), 2);
    }

    #[test]
    fn test_item_stack() {
        let mut item = InventoryItem::new("potion", "HP Potion")
            .with_quantity(1)
            .with_max_stack(10);
        item.add(5);
        assert_eq!(item.quantity, 6);
        item.add(10);
        assert_eq!(item.quantity, 10);
        assert!(item.is_full());
    }
}
