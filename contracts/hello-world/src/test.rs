#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let contract = env.register(InvoiceManagementContract, ());
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    (env, contract, seller, buyer)
}

#[test]
fn create_and_pay_updates_invoice_and_stats() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    let id = client.create_invoice(&seller, &buyer, &250, &String::from_str(&env, "Test invoice"));
    assert_eq!(id, 1);
    assert_eq!(env.auths().len(), 1);
    assert_eq!(env.auths()[0].0, seller);
    let invoice = client.get_invoice(&id);
    assert_eq!(invoice.amount, 250);
    assert_eq!(invoice.buyer, buyer);
    assert!(!invoice.is_paid);
    assert_eq!(client.get_stats().pending_invoices, 1);
    client.pay_invoice(&id, &buyer);
    assert_eq!(env.auths().len(), 1);
    assert_eq!(env.auths()[0].0, buyer);
    assert!(client.get_invoice(&id).is_paid);
    let stats = client.get_stats();
    assert_eq!(stats.total_invoices, 1);
    assert_eq!(stats.paid_invoices, 1);
    assert_eq!(stats.pending_invoices, 0);
    assert_eq!(stats.total_volume, 250);
}

#[test]
fn invalid_amounts_leave_records_and_statistics_unchanged() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    for amount in [0, -1, i128::MIN] {
        assert!(client.try_create_invoice(&seller, &buyer, &amount, &String::from_str(&env, "Invalid")).is_err());
        assert_eq!(client.get_stats().total_invoices, 0);
        assert_eq!(client.get_stats().pending_invoices, 0);
    }
    assert_eq!(client.create_invoice(&seller, &buyer, &1, &String::from_str(&env, "Valid")), 1);
}

#[test]
fn invoice_ids_are_sequential() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    assert_eq!(client.create_invoice(&seller, &buyer, &10, &String::from_str(&env, "First")), 1);
    assert_eq!(client.create_invoice(&seller, &buyer, &20, &String::from_str(&env, "Second")), 2);
    assert_eq!(client.get_stats().total_invoices, 2);
    assert_eq!(client.get_stats().pending_invoices, 2);
}

#[test]
fn another_buyer_cannot_mark_invoice_paid() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    let id = client.create_invoice(&seller, &buyer, &100, &String::from_str(&env, "Test"));
    let other = Address::generate(&env);
    assert!(client.try_pay_invoice(&id, &other).is_err());
    assert!(!client.get_invoice(&id).is_paid);
    assert_eq!(client.get_stats().pending_invoices, 1);
    assert_eq!(client.get_stats().total_volume, 0);
}

#[test]
fn paying_twice_does_not_double_count_volume() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    let id = client.create_invoice(&seller, &buyer, &100, &String::from_str(&env, "Test"));
    client.pay_invoice(&id, &buyer);
    assert!(client.try_pay_invoice(&id, &buyer).is_err());
    assert_eq!(client.get_stats().paid_invoices, 1);
    assert_eq!(client.get_stats().pending_invoices, 0);
    assert_eq!(client.get_stats().total_volume, 100);
}

#[test]
fn missing_invoice_cannot_be_paid() {
    let (env, contract, _, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    assert!(client.try_pay_invoice(&999, &buyer).is_err());
    assert_eq!(client.get_stats().paid_invoices, 0);
    assert_eq!(client.get_stats().total_volume, 0);
}

#[test]
fn invoice_creation_requires_seller_authorization() {
    let (env, contract, seller, buyer) = setup();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    assert!(client.try_create_invoice(&seller, &buyer, &100, &String::from_str(&env, "Test")).is_err());
    assert_eq!(client.get_stats().total_invoices, 0);
}

#[test]
fn marking_paid_requires_buyer_authorization() {
    let (env, contract, seller, buyer) = setup();
    env.mock_all_auths();
    let client = InvoiceManagementContractClient::new(&env, &contract);
    let id = client.create_invoice(&seller, &buyer, &100, &String::from_str(&env, "Test"));
    env.set_auths(&[]);
    assert!(client.try_pay_invoice(&id, &buyer).is_err());
    assert!(!client.get_invoice(&id).is_paid);
    assert_eq!(client.get_stats().pending_invoices, 1);
}
