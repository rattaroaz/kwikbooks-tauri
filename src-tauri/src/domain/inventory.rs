//! Quantity-on-hand is not modeled yet; reserved for future posting guards.

pub fn allow_sale_without_inventory_check(_item_id: Option<i64>) -> bool {
    true
}
