//! God method example - individual methods that are too complex

use std::collections::HashMap;

pub struct PaymentProcessor {
    transactions: HashMap<u64, Transaction>,
}

impl PaymentProcessor {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    /// This method does WAY too much - it's a god method
    pub fn process_complex_payment(
        &mut self,
        user_id: u64,
        amount: f64,
        currency: &str,
        payment_method: &str,
        billing_address: Address,
        shipping_address: Option<Address>,
        discount_code: Option<String>,
        loyalty_points: u64,
        split_payment: bool,
        save_payment_method: bool,
        send_receipt: bool,
        notification_preferences: NotificationSettings,
    ) -> Result<PaymentResult, String> {
        // Validate user
        if user_id == 0 {
            return Err("Invalid user ID".to_string());
        }

        // Validate amount
        if amount <= 0.0 {
            return Err("Invalid amount".to_string());
        }

        // Validate currency
        let valid_currencies = vec!["USD", "EUR", "GBP", "JPY"];
        if !valid_currencies.contains(&currency) {
            return Err("Unsupported currency".to_string());
        }

        // Validate payment method
        let valid_methods = vec!["credit_card", "debit_card", "paypal", "crypto"];
        if !valid_methods.contains(&payment_method) {
            return Err("Invalid payment method".to_string());
        }

        // Validate addresses
        if billing_address.country.is_empty() {
            return Err("Invalid billing address".to_string());
        }

        if billing_address.postal_code.is_empty() {
            return Err("Postal code required".to_string());
        }

        // Check if international
        let is_international = if let Some(ref ship_addr) = shipping_address {
            ship_addr.country != billing_address.country
        } else {
            false
        };

        // Calculate fees
        let mut total_amount = amount;
        let mut fees = 0.0;

        if payment_method == "credit_card" {
            fees += amount * 0.029 + 0.30;
        } else if payment_method == "paypal" {
            fees += amount * 0.034 + 0.30;
        } else if payment_method == "crypto" {
            fees += amount * 0.01;
        }

        if is_international {
            fees += amount * 0.015;
        }

        total_amount += fees;

        // Apply discount
        let mut discount_applied = 0.0;
        if let Some(code) = discount_code {
            if code == "SAVE10" {
                discount_applied = total_amount * 0.10;
            } else if code == "SAVE20" {
                discount_applied = total_amount * 0.20;
            } else if code == "SUMMER25" {
                discount_applied = total_amount * 0.25;
            } else if code == "VIP50" {
                discount_applied = total_amount * 0.50;
            } else {
                return Err("Invalid discount code".to_string());
            }
            total_amount -= discount_applied;
        }

        // Apply loyalty points
        let mut points_used = 0;
        if loyalty_points > 0 {
            let points_value = loyalty_points as f64 * 0.01;
            let max_discount = total_amount * 0.30;
            let points_discount = points_value.min(max_discount);
            total_amount -= points_discount;
            points_used = (points_discount / 0.01) as u64;
        }

        // Handle split payment
        let mut split_amounts = Vec::new();
        if split_payment {
            let num_splits = if total_amount > 1000.0 {
                4
            } else if total_amount > 500.0 {
                3
            } else if total_amount > 100.0 {
                2
            } else {
                1
            };

            let split_amount = total_amount / num_splits as f64;
            for i in 0..num_splits {
                split_amounts.push(split_amount);
            }
        }

        // Process payment
        let transaction_id = self.transactions.len() as u64 + 1;
        
        // Fraud detection
        if total_amount > 10000.0 {
            if payment_method == "crypto" {
                return Err("High-value crypto transactions require manual review".to_string());
            }
        }

        if is_international && total_amount > 5000.0 {
            return Err("High-value international transactions require verification".to_string());
        }

        // Risk scoring
        let mut risk_score = 0;
        if total_amount > 1000.0 {
            risk_score += 10;
        }
        if is_international {
            risk_score += 15;
        }
        if payment_method == "crypto" {
            risk_score += 20;
        }
        if discount_applied > 0.0 {
            risk_score += 5;
        }

        if risk_score > 40 {
            return Err("Transaction flagged as high risk".to_string());
        }

        // Create transaction record
        let transaction = Transaction {
            id: transaction_id,
            user_id,
            amount: total_amount,
            currency: currency.to_string(),
            payment_method: payment_method.to_string(),
            status: "pending".to_string(),
            fees,
            discount_applied,
            points_used,
        };

        self.transactions.insert(transaction_id, transaction);

        // Save payment method
        if save_payment_method {
            // Simulate saving payment method
            println!("Saving payment method for user {}", user_id);
        }

        // Send notifications
        if send_receipt {
            if notification_preferences.email {
                println!("Sending email receipt to user {}", user_id);
            }
            if notification_preferences.sms {
                println!("Sending SMS receipt to user {}", user_id);
            }
            if notification_preferences.push {
                println!("Sending push notification to user {}", user_id);
            }
        }

        // Update inventory
        println!("Updating inventory for transaction {}", transaction_id);

        // Update analytics
        println!("Recording analytics for transaction {}", transaction_id);

        // Log transaction
        println!(
            "Transaction {} processed: ${:.2} (fees: ${:.2}, discount: ${:.2})",
            transaction_id, total_amount, fees, discount_applied
        );

        Ok(PaymentResult {
            transaction_id,
            amount_charged: total_amount,
            points_used,
            discount_applied,
        })
    }

    pub fn get_transaction(&self, id: u64) -> Option<&Transaction> {
        self.transactions.get(&id)
    }
}

#[derive(Debug, Clone)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug)]
pub struct NotificationSettings {
    pub email: bool,
    pub sms: bool,
    pub push: bool,
}

#[derive(Debug)]
pub struct Transaction {
    pub id: u64,
    pub user_id: u64,
    pub amount: f64,
    pub currency: String,
    pub payment_method: String,
    pub status: String,
    pub fees: f64,
    pub discount_applied: f64,
    pub points_used: u64,
}

#[derive(Debug)]
pub struct PaymentResult {
    pub transaction_id: u64,
    pub amount_charged: f64,
    pub points_used: u64,
    pub discount_applied: f64,
}

