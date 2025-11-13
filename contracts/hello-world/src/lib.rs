#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store invoice details
#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    pub invoice_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: i128,
    pub description: String,
    pub created_at: u64,
    pub is_paid: bool,
    pub paid_at: u64,
}

// Structure to track overall invoice statistics
#[contracttype]
#[derive(Clone)]
pub struct InvoiceStats {
    pub total_invoices: u64,
    pub paid_invoices: u64,
    pub pending_invoices: u64,
    pub total_volume: i128,
}

// Mapping invoice_id to Invoice
#[contracttype]
pub enum InvoiceBook {
    Invoice(u64)
}

// Symbol for invoice counter
const INVOICE_COUNT: Symbol = symbol_short!("INV_CNT");
// Symbol for invoice statistics
const STATS: Symbol = symbol_short!("STATS");

#[contract]
pub struct InvoiceManagementContract;

#[contractimpl]
impl InvoiceManagementContract {
    
    // Function to create a new invoice
    pub fn create_invoice(
        env: Env, 
        seller: Address, 
        buyer: Address, 
        amount: i128, 
        description: String
    ) -> u64 {
        // Verify seller authorization
        seller.require_auth();
        
        // Get and increment invoice counter
        let mut invoice_count: u64 = env.storage().instance().get(&INVOICE_COUNT).unwrap_or(0);
        invoice_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new invoice
        let new_invoice = Invoice {
            invoice_id: invoice_count,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            description,
            created_at: timestamp,
            is_paid: false,
            paid_at: 0,
        };
        
        // Update statistics
        let mut stats = Self::get_stats(env.clone());
        stats.total_invoices += 1;
        stats.pending_invoices += 1;
        
        // Store invoice and update storage
        env.storage().instance().set(&InvoiceBook::Invoice(invoice_count), &new_invoice);
        env.storage().instance().set(&INVOICE_COUNT, &invoice_count);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Invoice Created - ID: {}, Amount: {}", invoice_count, amount);
        invoice_count
    }
    
    // Function to mark invoice as paid
    pub fn pay_invoice(env: Env, invoice_id: u64, buyer: Address) {
        // Verify buyer authorization
        buyer.require_auth();
        
        // Get invoice
        let mut invoice = Self::get_invoice(env.clone(), invoice_id);
        
        // Verify invoice exists and is not already paid
        if invoice.invoice_id == 0 {
            log!(&env, "Invoice not found!");
            panic!("Invoice not found!");
        }
        
        if invoice.is_paid {
            log!(&env, "Invoice already paid!");
            panic!("Invoice already paid!");
        }
        
        // Verify buyer matches
        if invoice.buyer != buyer {
            log!(&env, "Unauthorized: Not the invoice buyer!");
            panic!("Unauthorized buyer!");
        }
        
        // Mark invoice as paid
        let timestamp = env.ledger().timestamp();
        invoice.is_paid = true;
        invoice.paid_at = timestamp;
        
        // Update statistics
        let mut stats = Self::get_stats(env.clone());
        stats.paid_invoices += 1;
        stats.pending_invoices -= 1;
        stats.total_volume += invoice.amount;
        
        // Update storage
        env.storage().instance().set(&InvoiceBook::Invoice(invoice_id), &invoice);
        env.storage().instance().set(&STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Invoice Paid - ID: {}, Amount: {}", invoice_id, invoice.amount);
    }
    
    // Function to retrieve invoice details
    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        let key = InvoiceBook::Invoice(invoice_id);
        
        env.storage().instance().get(&key).unwrap_or(Invoice {
            invoice_id: 0,
            seller: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            buyer: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            amount: 0,
            description: String::from_str(&env, "Not Found"),
            created_at: 0,
            is_paid: false,
            paid_at: 0,
        })
    }
    
    // Function to retrieve invoice statistics
    pub fn get_stats(env: Env) -> InvoiceStats {
        env.storage().instance().get(&STATS).unwrap_or(InvoiceStats {
            total_invoices: 0,
            paid_invoices: 0,
            pending_invoices: 0,
            total_volume: 0,
        })
    }
}