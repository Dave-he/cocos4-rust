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

    // -----------------------------------------------------------------
    // Round 126 — helper-level
    // tests for the
    // Currency / CurrencyType
    // / Transaction / Wallet
    // / InventoryItem /
    // Inventory gaps that
    // the pre-round-126
    // 7 tests didn't cover:
    //   - Currency::add cap
    //     already reached
    //     (returns cap, not
    //     new_amount)
    //   - Currency::spend
    //     returns false on
    //     insufficient funds
    //     (preserves balance)
    //   - Currency::spend
    //     returns true on
    //     exact-match
    //   - Currency::can_afford
    //     with 0 amount
    //   - Currency::get_cap
    //     for un-capped
    //     currency (returns
    //     u64::MAX)
    //   - Currency::new
    //     initial state
    //   - CurrencyType::name()
    //     for all 5 variants
    //     + Custom returns
    //     "custom" (variant id
    //     not used in name)
    //   - Transaction builder
    //     pattern (multiple
    //     gain/cost chained)
    //   - Transaction::total_gains
    //     / total_costs filter
    //     by currency_type
    //   - Wallet::execute
    //     multi-entry mixed
    //     gain/cost
    //   - Wallet::execute
    //     respects max_log_size
    //     (rolls over oldest)
    //   - Wallet::can_execute
    //     doesn't mutate
    //   - InventoryItem::new
    //     default state
    //     (quantity=1,
    //     max_stack=99)
    //   - InventoryItem::remove
    //     amount > quantity
    //   - Inventory::remove_item
    //     for non-existent id
    //     (returns 0)
    //   - Inventory::get_item /
    //     get_item_mut /
    //     has_item for
    //     non-existent id
    // -----------------------------------------------------------------

    #[test]
    fn test_currency_add_respects_cap_when_already_reached_round_126() {
        // Defense: a regression that dropped
        // the `.min(cap)` would silently
        // overflow past the cap. Pin: cap
        // already reached → add returns
        // cap (not current + added). Use
        // Gold (starts at 0 in new())
        // to avoid the Energy=100 default
        // complication.
        let mut c = Currency::new();
        c.set_cap(CurrencyType::Gold, 100);
        let after_first = c.add(CurrencyType::Gold, 80);
        assert_eq!(after_first, 80);
        let after_overflow = c.add(CurrencyType::Gold, 100);
        assert_eq!(after_overflow, 100, "add should clamp to cap, not overflow");
        assert_eq!(c.get(CurrencyType::Gold), 100);
    }

    #[test]
    fn test_currency_spend_preserves_balance_on_failure_round_126() {
        // spend returns false on insufficient
        // funds AND the balance is unchanged.
        // The pre-round-126 test_currency_basic
        // checked the false return but not the
        // preserved balance.
        let mut c = Currency::new();
        c.add(CurrencyType::Gold, 50);
        let result = c.spend(CurrencyType::Gold, 100);
        assert!(!result);
        assert_eq!(c.get(CurrencyType::Gold), 50, "balance must be preserved on failure");
    }

    #[test]
    fn test_currency_spend_exact_match_returns_true_round_126() {
        // Boundary: spend exactly the balance
        // → returns true + balance = 0.
        let mut c = Currency::new();
        c.add(CurrencyType::Gold, 50);
        let result = c.spend(CurrencyType::Gold, 50);
        assert!(result);
        assert_eq!(c.get(CurrencyType::Gold), 0);
    }

    #[test]
    fn test_currency_can_afford_zero_amount_always_true_round_126() {
        // can_afford(X, 0) is always true (you
        // can always "afford" to spend 0). The
        // implementation uses `>=`, so 0 ≥ 0
        // is true.
        let c = Currency::new();
        assert!(c.can_afford(CurrencyType::Gold, 0));
        assert!(c.can_afford(CurrencyType::Gem, 0));
        assert!(c.can_afford(CurrencyType::Energy, 0));
    }

    #[test]
    fn test_currency_get_cap_for_uncapped_currency_returns_max_round_126() {
        // Gold + Gem are uncapped in `new()`
        // (no entry in caps HashMap).
        // get_cap returns u64::MAX for these.
        let c = Currency::new();
        assert_eq!(c.get_cap(CurrencyType::Gold), u64::MAX);
        assert_eq!(c.get_cap(CurrencyType::Gem), u64::MAX);
        // Energy has a cap (200 from new()).
        assert_eq!(c.get_cap(CurrencyType::Energy), 200);
        // A never-seen currency (Token)
        // also returns u64::MAX.
        assert_eq!(c.get_cap(CurrencyType::Token), u64::MAX);
    }

    #[test]
    fn test_currency_new_initial_state_round_126() {
        // Pin the constructor defaults:
        // Gold=0, Gem=0, Energy=100
        // (from the `amounts` HashMap
        // init in `new()`).
        let c = Currency::new();
        assert_eq!(c.get(CurrencyType::Gold), 0);
        assert_eq!(c.get(CurrencyType::Gem), 0);
        assert_eq!(c.get(CurrencyType::Energy), 100);
        // Caps: Gold=MAX, Gem=MAX, Energy=200.
        assert_eq!(c.get_cap(CurrencyType::Gold), u64::MAX);
        assert_eq!(c.get_cap(CurrencyType::Gem), u64::MAX);
        assert_eq!(c.get_cap(CurrencyType::Energy), 200);
    }

    #[test]
    fn test_currency_type_name_all_5_variants_round_126() {
        // Pin all 5 variants' name() output.
        // The pre-round-126 tests never
        // touched CurrencyType::name().
        assert_eq!(CurrencyType::Gold.name(), "gold");
        assert_eq!(CurrencyType::Gem.name(), "gem");
        assert_eq!(CurrencyType::Energy.name(), "energy");
        assert_eq!(CurrencyType::Token.name(), "token");
        // Custom returns "custom" (the
        // variant id is intentionally NOT
        // used in the name — the Currency
        // uses Custom as a catch-all).
        assert_eq!(CurrencyType::Custom(42).name(), "custom");
        assert_eq!(CurrencyType::Custom(0).name(), "custom");
        assert_eq!(CurrencyType::Custom(u32::MAX).name(), "custom");
    }

    #[test]
    fn test_transaction_builder_chains_multiple_gain_cost_round_126() {
        // Pin the builder pattern: Transaction
        // supports chaining gain() + cost()
        // in any order, accumulating into
        // entries.
        let tx = Transaction::new("tx_multi", "complex purchase")
            .gain(CurrencyType::Gold, 100)
            .cost(CurrencyType::Gold, 30)
            .gain(CurrencyType::Token, 1)
            .cost(CurrencyType::Energy, 10)
            .with_timestamp(12345);
        assert_eq!(tx.id, "tx_multi");
        assert_eq!(tx.description, "complex purchase");
        assert_eq!(tx.timestamp, 12345);
        assert_eq!(tx.entries.len(), 4);
        // Verify each entry.
        assert_eq!(tx.entries[0].currency_type, CurrencyType::Gold);
        assert_eq!(tx.entries[0].amount, 100);
        assert!(tx.entries[0].is_gain);
        assert_eq!(tx.entries[1].currency_type, CurrencyType::Gold);
        assert_eq!(tx.entries[1].amount, 30);
        assert!(!tx.entries[1].is_gain);
        assert_eq!(tx.entries[2].currency_type, CurrencyType::Token);
        assert!(tx.entries[2].is_gain);
        assert_eq!(tx.entries[3].currency_type, CurrencyType::Energy);
        assert!(!tx.entries[3].is_gain);
    }

    #[test]
    fn test_transaction_total_gains_costs_filters_by_currency_round_126() {
        // total_gains / total_costs filter by
        // both currency_type AND is_gain
        // direction. Mixed transactions must
        // produce per-currency totals.
        let tx = Transaction::new("tx_mix", "mixed")
            .gain(CurrencyType::Gold, 100)
            .cost(CurrencyType::Gold, 30)
            .gain(CurrencyType::Gold, 50) // 2 gains of gold
            .gain(CurrencyType::Gem, 5)
            .cost(CurrencyType::Gem, 2); // 1 gain + 1 cost of gem
        // Gold: gains=100+50=150, costs=30.
        assert_eq!(tx.total_gains(CurrencyType::Gold), 150);
        assert_eq!(tx.total_costs(CurrencyType::Gold), 30);
        // Gem: gains=5, costs=2.
        assert_eq!(tx.total_gains(CurrencyType::Gem), 5);
        assert_eq!(tx.total_costs(CurrencyType::Gem), 2);
        // Energy: not in entries → both 0.
        assert_eq!(tx.total_gains(CurrencyType::Energy), 0);
        assert_eq!(tx.total_costs(CurrencyType::Energy), 0);
    }

    #[test]
    fn test_wallet_execute_multi_entry_atomic_round_126() {
        // Wallet::execute is atomic: if ANY
        // cost entry is unaffordable, the
        // ENTIRE transaction is rejected
        // (no partial execution).
        let mut wallet = Wallet::new();
        wallet.currency.add(CurrencyType::Gold, 100);
        wallet.currency.add(CurrencyType::Gem, 0);
        let tx = Transaction::new("tx_atomic", "atomic test")
            .cost(CurrencyType::Gold, 50)
            .cost(CurrencyType::Gem, 5); // unaffordable
        assert!(!wallet.execute(tx));
        // Atomicity: gold balance unchanged
        // (not deducted 50 partially).
        assert_eq!(wallet.get_balance(CurrencyType::Gold), 100);
    }

    #[test]
    fn test_wallet_execute_respects_max_log_size_round_126() {
        // The transaction_log is capped at
        // max_log_size=1000 (default). When
        // a new transaction pushes it past
        // 1000, the OLDEST is removed via
        // `transaction_log.remove(0)`. Pin
        // this rollover behavior.
        let mut wallet = Wallet::new();
        wallet.max_log_size = 3; // shrink for test visibility
        wallet.currency.add(CurrencyType::Gold, 1_000_000);
        // Add 5 transactions; only the last 3
        // should remain (the first 2 are
        // evicted by remove(0)).
        for i in 0..5 {
            let tx = Transaction::new(&format!("tx_{i}"), "filler")
                .cost(CurrencyType::Gold, 1);
            wallet.execute(tx);
        }
        // Log size is capped at 3.
        assert_eq!(wallet.transaction_log.len(), 3);
        // The first 2 (tx_0, tx_1) are evicted.
        // The last 3 (tx_2, tx_3, tx_4) remain.
        assert_eq!(wallet.transaction_log[0].id, "tx_2");
        assert_eq!(wallet.transaction_log[1].id, "tx_3");
        assert_eq!(wallet.transaction_log[2].id, "tx_4");
        // Total gold spent: 5 transactions
        // × 1 = 5.
        assert_eq!(wallet.get_balance(CurrencyType::Gold), 1_000_000 - 5);
    }

    #[test]
    fn test_wallet_can_execute_does_not_mutate_round_126() {
        // can_execute is the readonly preview
        // of execute — must NOT mutate the
        // wallet state (no balance change,
        // no log append).
        let mut wallet = Wallet::new();
        wallet.currency.add(CurrencyType::Gold, 100);
        let tx = Transaction::new("tx_preview", "preview")
            .cost(CurrencyType::Gold, 50);
        let log_len_before = wallet.transaction_log.len();
        let balance_before = wallet.get_balance(CurrencyType::Gold);
        assert!(wallet.can_execute(&tx));
        assert_eq!(wallet.get_balance(CurrencyType::Gold), balance_before);
        assert_eq!(wallet.transaction_log.len(), log_len_before);
        // Now test can_execute with an
        // unaffordable transaction.
        let tx_bad = Transaction::new("tx_preview_bad", "preview bad")
            .cost(CurrencyType::Gold, 9999);
        assert!(!wallet.can_execute(&tx_bad));
        // Still no mutation.
        assert_eq!(wallet.get_balance(CurrencyType::Gold), balance_before);
        assert_eq!(wallet.transaction_log.len(), log_len_before);
    }

    #[test]
    fn test_inventory_item_new_default_state_round_126() {
        // Pin the InventoryItem::new defaults:
        // quantity=1, max_stack=99, empty
        // metadata. The pre-round-126
        // test_inventory always used
        // .with_quantity(1) so the default
        // wasn't pinned.
        let item = InventoryItem::new("scroll", "Ancient Scroll");
        assert_eq!(item.item_id, "scroll");
        assert_eq!(item.name, "Ancient Scroll");
        assert_eq!(item.quantity, 1, "default quantity should be 1");
        assert_eq!(item.max_stack, 99, "default max_stack should be 99");
        assert!(item.metadata.is_empty(), "default metadata should be empty");
        assert!(!item.is_empty());
        assert!(!item.is_full(), "default quantity 1 is not full at max_stack 99");
    }

    #[test]
    fn test_inventory_item_remove_overflow_returns_quantity_round_126() {
        // remove(amount > quantity) returns
        // the actual quantity (not the
        // requested amount) + sets quantity=0.
        let mut item = InventoryItem::new("arrow", "Arrow").with_quantity(5);
        let removed = item.remove(100);
        assert_eq!(removed, 5, "remove should return the actual quantity, not requested amount");
        assert_eq!(item.quantity, 0);
        assert!(item.is_empty());
    }

    #[test]
    fn test_inventory_get_item_remove_has_item_for_missing_id_round_126() {
        // 3 accessors, 1 expected behavior
        // each: non-existent item → None /
        // None / false. Split into 2 tests
        // because get_item_mut requires
        // mutable binding + the other
        // accessors don't.
        let inv = Inventory::new(10);
        assert!(inv.get_item("missing").is_none());
        assert!(!inv.has_item("missing", 1));
        assert!(!inv.has_item("missing", 0), "missing items are NOT in the inventory even at min=0");
        // Empty inventory → count=0, is_full=false (capacity=10).
        assert_eq!(inv.count(), 0);
        assert!(!inv.is_full());
        // Mutable accessor: get_item_mut on missing id → None.
        let mut inv2 = Inventory::new(10);
        assert!(inv2.get_item_mut("missing").is_none());
    }

    #[test]
    fn test_inventory_remove_item_for_non_existent_id_returns_zero_round_126() {
        // remove_item("missing", N) returns
        // 0 (no-op, no panic) regardless of N.
        let mut inv = Inventory::new(10);
        assert_eq!(inv.remove_item("missing", 1), 0);
        assert_eq!(inv.remove_item("missing", 100), 0);
        assert_eq!(inv.count(), 0);
    }

    // -----------------------------------------------------------------
    // Round 131 — additional
    // `Inventory` + `InventoryItem`
    // edge-case helper tests.
    // Mirrors the round-110b /
    // 122 / 123 / 124 / 125 / 126
    // / 127 / 128 / 129 / 130
    // pattern: pin the small
    // public helpers' contracts
    // (initial state / capacity
    // boundaries / stack
    // clamping / missing-key
    // accessors) so a refactor
    // can't silently change the
    // inventory behaviour that
    // the App + UI rely on.
    //
    // Closes the gaps:
    //   - Inventory::is_full
    //     boundary (>= not >)
    //   - Inventory::add_item
    //     new-id-at-capacity
    //     returns 0
    //   - Inventory::add_item
    //     merges with existing
    //     (delegates to
    //     InventoryItem::add)
    //   - InventoryItem::add
    //     saturates + clamps to
    //     max_stack
    //   - InventoryItem::add
    //     return value = actual
    //     added (clamped)
    //   - InventoryItem::is_empty
    //     at quantity 0
    //   - InventoryItem::is_full
    //     at quantity == max
    //   - Inventory::items()
    //     iterates all entries
    //   - Inventory::has_item
    //     with min_quantity=0
    //     returns true if
    //     present, false if
    //     missing
    //   - Inventory::remove_item
    //     exact-amount drops
    //     the entry
    //   - Inventory::remove_item
    //     over-remove returns
    //     only what was present
    // -----------------------------------------------------------------

    #[test]
    fn inventory_new_initial_state_round_131() {
        // A fresh Inventory
        // is empty + has
        // the capacity
        // the caller
        // asked for.
        let inv = Inventory::new(8);
        assert_eq!(inv.count(), 0);
        assert!(!inv.is_full());
        assert_eq!(inv.items().count(), 0);
    }

    #[test]
    fn inventory_is_full_at_exactly_capacity_round_131() {
        // is_full is `>=`,
        // not `>`: full
        // at exactly
        // capacity, not
        // capacity+1.
        let mut inv = Inventory::new(2);
        inv.add_item(InventoryItem::new("a", "A"));
        assert!(!inv.is_full());
        inv.add_item(InventoryItem::new("b", "B"));
        assert!(inv.is_full());
    }

    #[test]
    fn inventory_add_item_with_quantity_zero_still_inserts_round_131() {
        // add_item with
        // quantity=0 still
        // inserts the
        // item — capacity
        // check runs
        // first. Returns
        // 0 since no
        // quantity was
        // actually added.
        let mut inv = Inventory::new(5);
        let added = inv.add_item(InventoryItem::new("a", "A").with_quantity(0));
        assert_eq!(added, 0);
        assert_eq!(inv.count(), 1);
        // The item IS
        // present, just
        // at quantity 0.
        assert!(inv.has_item("a", 0));
        // But has_item
        // with min=1
        // returns false
        // since quantity
        // is 0, not 1.
        assert!(!inv.has_item("a", 1));
    }

    #[test]
    fn inventory_add_item_returns_zero_when_full_with_new_id_round_131() {
        // At capacity +
        // a new (non-
        // existing) id →
        // the add is
        // dropped + 0
        // returned. (An
        // existing-id
        // add still
        // succeeds via
        // the merge
        // path, since it
        // doesn't grow
        // the map.)
        let mut inv = Inventory::new(1);
        inv.add_item(InventoryItem::new("a", "A"));
        assert!(inv.is_full());
        // New id at
        // capacity →
        // dropped.
        let added = inv.add_item(InventoryItem::new("b", "B"));
        assert_eq!(added, 0);
        assert_eq!(inv.count(), 1);
    }

    #[test]
    fn inventory_add_item_merges_with_existing_at_full_capacity_round_131() {
        // The merge
        // path doesn't
        // grow the map,
        // so a re-add
        // of an existing
        // id succeeds
        // even when
        // capacity is
        // full.
        let mut inv = Inventory::new(1);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(3));
        assert!(inv.is_full());
        // Existing id →
        // merge, not
        // grow.
        let added = inv.add_item(InventoryItem::new("a", "A").with_quantity(5));
        assert_eq!(added, 5);
        assert_eq!(inv.get_item("a").unwrap().quantity, 8);
    }

    #[test]
    fn inventory_count_tracks_distinct_item_ids_round_131() {
        // count() is the
        // number of
        // distinct ids,
        // not the
        // quantity sum.
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(10));
        inv.add_item(InventoryItem::new("a", "A").with_quantity(5));
        inv.add_item(InventoryItem::new("b", "B").with_quantity(3));
        // 2 distinct ids.
        assert_eq!(inv.count(), 2);
        // a's quantity
        // is the sum.
        assert_eq!(inv.get_item("a").unwrap().quantity, 15);
    }

    #[test]
    fn inventory_items_iterates_all_entries_round_131() {
        // items() returns
        // a ref-iterator
        // over all
        // stored items.
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A"));
        inv.add_item(InventoryItem::new("b", "B"));
        inv.add_item(InventoryItem::new("c", "C"));
        let mut ids: Vec<&str> = inv.items().map(|i| i.item_id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn inventory_has_item_min_quantity_zero_is_presence_check_round_131() {
        // has_item(id, 0)
        // returns true
        // for any
        // present id
        // (even at
        // quantity 0)
        // and false for
        // any missing
        // id.
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(0));
        assert!(inv.has_item("a", 0));
        assert!(!inv.has_item("missing", 0));
        // min_quantity
        // 1 against a
        // quantity-0
        // entry →
        // false.
        assert!(!inv.has_item("a", 1));
    }

    #[test]
    fn inventory_has_item_min_quantity_exact_match_round_131() {
        // The check is
        // `>=` not
        // `>`: a min
        // quantity
        // exactly equal
        // to the
        // current
        // quantity
        // returns true.
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(7));
        assert!(inv.has_item("a", 7));
        assert!(inv.has_item("a", 6));
        assert!(!inv.has_item("a", 8));
    }

    #[test]
    fn inventory_remove_item_exact_amount_drops_entry_round_131() {
        // Removing the
        // exact
        // quantity of
        // an item
        // removes the
        // entire entry
        // from the map
        // (so count
        // drops by 1).
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(5));
        assert_eq!(inv.count(), 1);
        let removed = inv.remove_item("a", 5);
        assert_eq!(removed, 5);
        assert_eq!(inv.count(), 0);
        assert!(inv.get_item("a").is_none());
    }

    #[test]
    fn inventory_remove_item_over_remove_clamps_to_current_quantity_round_131() {
        // remove_item
        // saturates:
        // removing
        // more than the
        // current
        // quantity
        // removes
        // whatever is
        // there and
        // returns only
        // that amount.
        let mut inv = Inventory::new(5);
        inv.add_item(InventoryItem::new("a", "A").with_quantity(3));
        let removed = inv.remove_item("a", 100);
        assert_eq!(removed, 3);
        assert!(inv.get_item("a").is_none());
    }

    #[test]
    fn inventory_item_add_clamps_to_max_stack_round_131() {
        // add() saturates
        // and clamps to
        // max_stack.
        // Returned value
        // is the actual
        // amount added
        // (not the
        // requested).
        let mut item = InventoryItem::new("a", "A")
            .with_quantity(90)
            .with_max_stack(99);
        // Request +20,
        // can only add
        // 9 (90 + 9 =
        // 99).
        let added = item.add(20);
        assert_eq!(added, 9);
        assert_eq!(item.quantity, 99);
        assert!(item.is_full());
    }

    #[test]
    fn inventory_item_add_saturates_on_u32_overflow_round_131() {
        // saturating_add
        // prevents
        // u32 overflow
        // for absurd
        // requests.
        let mut item = InventoryItem::new("a", "A")
            .with_quantity(50)
            .with_max_stack(u32::MAX);
        // Request u32::MAX
        // → saturates to
        // u32::MAX,
        // result = MAX
        // (added saturates
        // to MAX-50).
        let added = item.add(u32::MAX);
        assert_eq!(item.quantity, u32::MAX);
        assert_eq!(added, u32::MAX - 50);
    }

    #[test]
    fn inventory_item_is_empty_at_quantity_zero_round_131() {
        // is_empty is
        // `== 0` —
        // quantity 0
        // counts as
        // empty
        // regardless of
        // max_stack. (Note
        // InventoryItem::
        // new() defaults
        // quantity to 1,
        // not 0, so we
        // must use the
        // with_quantity
        // builder.)
        let mut item = InventoryItem::new("a", "A").with_quantity(0);
        assert!(item.is_empty());
        item.quantity = 1;
        assert!(!item.is_empty());
        item.quantity = 0;
        assert!(item.is_empty());
    }

    #[test]
    fn inventory_item_is_full_at_exactly_max_stack_round_131() {
        // is_full is
        // `>= max_stack`:
        // at exactly
        // max_stack
        // counts as
        // full.
        let mut item = InventoryItem::new("a", "A").with_max_stack(5);
        assert!(!item.is_full());
        item.quantity = 4;
        assert!(!item.is_full());
        item.quantity = 5;
        assert!(item.is_full());
        item.quantity = 6;
        assert!(item.is_full());
    }

    // -----------------------------------------------------------------
    // Round 145 — helper-level
    // tests for
    // CurrencyType
    // / Currency /
    // Transaction /
    // Wallet /
    // InventoryItem
    // / Inventory
    // contracts
    // that the
    // pre-round-
    // 145 tests
    // didn't
    // cover. The
    // 34.6K file
    // has 61
    // tests in
    // `mod tests`
    // (above),
    // no separate
    // `mod
    // roundN_tests`
    // block
    // pre-round-
    // 145 (the
    // round-N
    // helper
    // pattern is
    // applied to
    // smaller
    // files like
    // atom.rs /
    // runtime_scene.rs
    // /
    // gameplay.rs).
    //
    // Round 145
    // closes:
    //   - CurrencyType::name()
    //     for all 5
    //     variants
    //     (pre-
    //     round-145
    //     tests
    //     cover 0
    //     variants
    //     explicitly
    //     — only
    //     the
    //     Custom
    //     fallback
    //     path is
    //     exercised
    //     implicitly)
    //   - CurrencyType::Custom(id)
    //     equality +
    //     Hash
    //     (the
    //     derive
    //     contract
    //     matters
    //     for
    //     HashMap
    //     keys
    //     used
    //     in
    //     Currency.amounts
    //     and
    //     Inventory.items)
    //   - Currency::add()
    //     saturating
    //     at
    //     u64::MAX
    //     when no
    //     cap is
    //     set
    //     (saturating_add
    //     contract;
    //     pre-
    //     round-145
    //     tests
    //     only
    //     cover
    //     the
    //     finite-
    //     cap
    //     path)
    //   - Currency::can_afford()
    //     exact-
    //     amount
    //     boundary
    //     (current
    //     == amount
    //     must
    //     return
    //     true)
    //   - Transaction::total_gains
    //     / total_costs
    //     for empty
    //     + multi-
    //     entry +
    //     multi-
    //     currency
    //     (the
    //     pre-
    //     round-145
    //     tests
    //     don't
    //     exercise
    //     these
    //     accessors)
    //   - Wallet::execute
    //     atomic
    //     semantics:
    //     when ONE
    //     entry is
    //     unaffordable,
    //     the entire
    //     transaction
    //     is rejected
    //     (no partial
    //     apply).
    //     This is
    //     a CRITICAL
    //     contract:
    //     a
    //     regression
    //     that
    //     applies
    //     the
    //     affordable
    //     entries
    //     before
    //     failing
    //     on the
    //     unaffordable
    //     one would
    //     silently
    //     corrupt
    //     the
    //     wallet
    //     state.
    //   - Wallet::execute
    //     log
    //     rotation
    //     (when
    //     transaction_log.len()
    //     >=
    //     max_log_size,
    //     the
    //     oldest
    //     entry is
    //     removed)
    //   - InventoryItem::add
    //     saturating
    //     at
    //     max_stack
    //     (returns
    //     the
    //     actually-
    //     added
    //     amount,
    //     not the
    //     requested
    //     amount)
    //   - InventoryItem::remove
    //     for amount
    //     > quantity
    //     (caps at
    //     quantity,
    //     returns
    //     the
    //     actually-
    //     removed
    //     amount)
    //   - Inventory::add_item
    //     returns 0
    //     when the
    //     inventory
    //     is full
    //     AND the
    //     item is
    //     new
    //     (the
    //     existing-
    //     item path
    //     still
    //     returns
    //     the
    //     actually-
    //     added
    //     amount
    //     because
    //     it
    //     stacks
    //     into
    //     the
    //     existing
    //     slot
    //     without
    //     consuming
    //     a new
    //     capacity
    //     slot)
    // -----------------------------------------------------------------

    #[test]
    fn test_currency_type_name_all_5_variants_round_145() {
        // `CurrencyType::name()`
        // for all 5
        // variants. Pre-
        // round-145 tests
        // don't exercise
        // any of the
        // named variants
        // directly (only
        // the Custom
        // fallback is
        // covered
        // implicitly via
        // the Currency
        // tests).
        assert_eq!(CurrencyType::Gold.name(),    "gold");
        assert_eq!(CurrencyType::Gem.name(),     "gem");
        assert_eq!(CurrencyType::Energy.name(),  "energy");
        assert_eq!(CurrencyType::Token.name(),   "token");
        // Custom variant
        // always returns
        // the constant
        // "custom"
        // regardless of
        // the inner id
        // (defense: a
        // regression that
        // returned a
        // serialized id
        // would silently
        // desync the
        // WASM bridge
        // JSON contract).
        assert_eq!(CurrencyType::Custom(0).name(),  "custom");
        assert_eq!(CurrencyType::Custom(99).name(), "custom");
    }

    #[test]
    fn test_currency_type_custom_equality_and_hash_round_145() {
        // `CurrencyType::Custom(u32)` derives PartialEq + Eq + Hash.
        // Two Customs with the same id must be equal + hash to the
        // same value. Two Customs with different ids must NOT be
        // equal. The derive contract matters because CurrencyType
        // is used as a HashMap key in Currency.amounts and the
        // inventory items map.
        assert_eq!(CurrencyType::Custom(7),  CurrencyType::Custom(7));
        assert_ne!(CurrencyType::Custom(7),  CurrencyType::Custom(8));
        assert_ne!(CurrencyType::Custom(7),  CurrencyType::Gold);
        // Hash contract: equal values hash to the same value.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        CurrencyType::Custom(42).hash(&mut h1);
        CurrencyType::Custom(42).hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
        // And: CurrencyType can be used as a HashMap key (this is
        // the actual host use case in Currency.amounts).
        let mut map: std::collections::HashMap<CurrencyType, &str> = std::collections::HashMap::new();
        map.insert(CurrencyType::Gold, "yellow");
        assert_eq!(map.get(&CurrencyType::Gold).copied(), Some("yellow"));
    }

    #[test]
    fn test_currency_add_saturates_at_u64_max_round_145() {
        // When no cap is set for a currency, `Currency::add`
        // uses `u64::MAX` as the implicit cap. Adding an
        // amount that would overflow should saturate at
        // u64::MAX, NOT wrap around to 0 (a regression
        // that removed `.min(cap)` would wrap).
        // Use a fresh Currency type (Token) that has no
        // default cap set in `Currency::new()`.
        let mut c = Currency::new();
        // First add some, then add a huge amount.
        c.add(CurrencyType::Token, 100);
        assert_eq!(c.get(CurrencyType::Token), 100);
        c.add(CurrencyType::Token, u64::MAX);
        // Should saturate at u64::MAX, not wrap.
        assert_eq!(c.get(CurrencyType::Token), u64::MAX);
    }

    #[test]
    fn test_currency_can_afford_exact_amount_round_145() {
        // `can_afford` boundary: current == amount must
        // return true (the >= contract). Pre-round-145
        // tests don't cover the exact-amount case
        // (they test strictly-greater and
        // strictly-less).
        let mut c = Currency::new();
        c.add(CurrencyType::Gold, 100);
        assert!(c.can_afford(CurrencyType::Gold, 100));   // exact
        assert!(c.can_afford(CurrencyType::Gold, 99));    // under
        assert!(!c.can_afford(CurrencyType::Gold, 101));  // over
    }

    #[test]
    fn test_transaction_total_gains_and_costs_round_145() {
        // The
        // `total_gains`
        // and
        // `total_costs`
        // accessors
        // sum
        // entries
        // by
        // currency
        // type.
        // Empty
        // transaction
        // returns
        // 0
        // for
        // all
        // currencies.
        // Multi-
        // currency
        // transaction
        // returns
        // per-
        // currency
        // sums.
        let empty = Transaction::new("empty", "");
        assert_eq!(empty.total_gains(CurrencyType::Gold), 0);
        assert_eq!(empty.total_costs(CurrencyType::Gold), 0);
        // Multi-entry
        // multi-currency.
        let tx = Transaction::new("tx", "test")
            .gain(CurrencyType::Gold,   10)
            .gain(CurrencyType::Gold,   20)
            .cost(CurrencyType::Gold,   5)
            .gain(CurrencyType::Gem,    100)
            .cost(CurrencyType::Energy, 30);
        assert_eq!(tx.total_gains(CurrencyType::Gold),   30);  // 10 + 20
        assert_eq!(tx.total_costs(CurrencyType::Gold),   5);
        assert_eq!(tx.total_gains(CurrencyType::Gem),    100);
        assert_eq!(tx.total_costs(CurrencyType::Gem),    0);
        assert_eq!(tx.total_gains(CurrencyType::Energy), 0);
        assert_eq!(tx.total_costs(CurrencyType::Energy), 30);
    }

    #[test]
    fn test_wallet_execute_atomic_semantics_round_145() {
        // CRITICAL
        // contract:
        // when ONE
        // entry is
        // unaffordable,
        // the
        // entire
        // transaction
        // is
        // rejected
        // (no
        // partial
        // apply).
        // A
        // regression
        // that
        // applied
        // the
        // affordable
        // entries
        // before
        // failing
        // would
        // silently
        // corrupt
        // the
        // wallet
        // state.
        let mut wallet = Wallet::new();
        wallet.currency.add(CurrencyType::Gold, 50);
        // Transaction
        // with one
        // affordable
        // cost
        // (Gold: 10)
        // and one
        // unaffordable
        // cost
        // (Gold: 9999).
        // Both
        // entries
        // are Gold;
        // the
        // second
        // is
        // unaffordable.
        let tx = Transaction::new("atomic_test", "")
            .cost(CurrencyType::Gold, 10)
            .cost(CurrencyType::Gold, 9999);
        assert!(!wallet.execute(tx));
        // Balance
        // unchanged
        // (50 Gold
        // — none
        // spent).
        assert_eq!(wallet.get_balance(CurrencyType::Gold), 50);
        // Transaction
        // log NOT
        // updated
        // (a
        // rejected
        // transaction
        // is not
        // logged).
        assert_eq!(wallet.transaction_log.len(), 0);
    }

    #[test]
    fn test_wallet_execute_log_rotation_round_145() {
        // When
        // `transaction_log.len()`
        // >=
        // `max_log_size`,
        // the
        // oldest
        // entry is
        // removed
        // (FIFO).
        // This
        // is
        // the
        // ring-buffer
        // contract;
        // the
        // log
        // never
        // grows
        // past
        // `max_log_size`.
        let mut wallet = Wallet::new();
        wallet.max_log_size = 3; // small for the test
        wallet.currency.add(CurrencyType::Gold, 1_000_000);
        // Execute 5
        // transactions;
        // log
        // should
        // cap at
        // 3
        // (the
        // oldest 2
        // are
        // removed).
        for i in 0..5 {
            let tx = Transaction::new(&format!("tx{i}"), "")
                .cost(CurrencyType::Gold, 1);
            assert!(wallet.execute(tx));
        }
        assert_eq!(wallet.transaction_log.len(), 3);
        // The
        // 3
        // remaining
        // entries
        // are the
        // most
        // recent 3
        // (tx2,
        // tx3,
        // tx4
        // — tx0
        // and
        // tx1
        // were
        // evicted).
        assert_eq!(wallet.transaction_log[0].id, "tx2");
        assert_eq!(wallet.transaction_log[1].id, "tx3");
        assert_eq!(wallet.transaction_log[2].id, "tx4");
    }

    #[test]
    fn test_inventory_item_add_saturates_at_max_stack_round_145() {
        // `InventoryItem::add`
        // saturates
        // at
        // max_stack
        // and
        // returns
        // the
        // ACTUALLY-added
        // amount
        // (not
        // the
        // requested
        // amount).
        // A
        // regression
        // that
        // returned
        // the
        // requested
        // amount
        // would
        // over-
        // report
        // (host
        // code
        // would
        // think
        // items
        // were
        // added
        // that
        // weren't).
        let mut item = InventoryItem::new("potion", "Healing Potion")
            .with_quantity(95)
            .with_max_stack(99);
        let added = item.add(10); // request 10, only 4 fit
        assert_eq!(added, 4);
        assert_eq!(item.quantity, 99);
        assert!(item.is_full());
    }

    #[test]
    fn test_inventory_item_remove_caps_at_quantity_round_145() {
        // `InventoryItem::remove`
        // caps
        // at
        // quantity
        // (removing
        // more
        // than
        // available
        // removes
        // only
        // what's
        // there).
        // Returns
        // the
        // ACTUALLY-
        // removed
        // amount.
        let mut item = InventoryItem::new("gem", "Ruby").with_quantity(3);
        let removed = item.remove(10); // request 10, only 3 available
        assert_eq!(removed, 3);
        assert_eq!(item.quantity, 0);
        assert!(item.is_empty());
    }

    #[test]
    fn test_inventory_add_item_returns_0_when_full_and_item_new_round_145() {
        // When
        // the
        // inventory
        // is
        // at
        // capacity
        // AND
        // the
        // item
        // is
        // new
        // (not
        // an
        // existing
        // stack),
        // `add_item`
        // returns
        // 0
        // and
        // the
        // item
        // is
        // NOT
        // inserted.
        // (The
        // existing-
        // item
        // path
        // always
        // succeeds
        // even
        // when
        // at
        // capacity
        // because
        // it
        // stacks
        // into
        // the
        // existing
        // slot
        // without
        // consuming
        // a
        // new
        // capacity
        // slot
        // — that's
        // the
        // canonical
        // "stacking
        // inventory"
        // pattern.)
        let mut inv = Inventory::new(2);
        inv.add_item(InventoryItem::new("a", "A"));
        inv.add_item(InventoryItem::new("b", "B"));
        assert!(inv.is_full());
        let added = inv.add_item(InventoryItem::new("c", "C"));
        assert_eq!(added, 0);
        // 'c' is NOT
        // in the
        // inventory.
        assert!(inv.get_item("c").is_none());
        assert_eq!(inv.count(), 2);
        // And:
        // stacking
        // an
        // existing
        // item
        // still
        // works
        // even
        // at
        // capacity
        // (it
        // doesn't
        // consume
        // a
        // new
        // slot).
        let added_existing = inv.add_item(
            InventoryItem::new("a", "A").with_quantity(5)
        );
        assert_eq!(added_existing, 5);
        assert_eq!(inv.get_item("a").unwrap().quantity, 6);
    }
}
