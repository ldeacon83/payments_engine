use std::collections::HashMap;
use crate::client::Client;
use crate::transaction::{Transaction, TransactionType::*, TransactionType};
use std::{fmt, io, error, error::Error, default::Default};


type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
enum LedgerError{
    MissingClient(u16),
    MissingTransaction(u32),
    MissingTransactionAmount(u32),
}

impl Error for LedgerError {}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            LedgerError::MissingClient(id) => write!(f, "Missing client for id {}", id),
            LedgerError::MissingTransaction(id) => write!(f, "Missing transaction for id {}", id),
            LedgerError::MissingTransactionAmount(id) => write!(f, "Missing transaction amaount for id {}", id),
        }
    }
}

#[derive(Default)]
pub struct Ledger {
    transaction_table: HashMap<u32,Transaction>,
    client_table: HashMap<u16, Client>,
}

impl Ledger {
    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<()>{
        //Get or make new client
        
        match tx.tx_type() {
            deposit => {
                self.init_client(tx.client());
                let c = self.get_client(tx.client())?;
                c.deposit(tx.amount().unwrap())?;  
                self.transaction_table.insert(*tx.tx(), *tx);
            },
            withdrawal => {
                let c = self.get_client(tx.client())?;
                c.deposit(-tx.amount().unwrap())?;
                self.transaction_table.insert(*tx.tx(), *tx);
            },
            dispute => {
                match self.transaction_table.get(tx.tx()){
                    Some(ref_tx) => {
                        if *ref_tx.tx_type() == deposit {
                            match ref_tx.amount() {
                                Some(a) => {
                                    let c = self.get_client(tx.client())?;
                                    c.hold(a)?;
                                },
                                None => return Err(Box::new(LedgerError::MissingTransactionAmount(*tx.tx())))
                            }
                        }
                    },
                    //Assume client has made a mistake of tx does not exist
                    None => return Ok(())
                }
            },
            resolve => {
                let (amount, tx_type) = self.get_tx_amount_type(tx.tx())?;
                if tx_type == deposit {
                    let c = self.get_client(tx.client())?;
                    c.hold(-amount)?;
                }
            },
            chargeback => {
                let (amount, tx_type) = self.get_tx_amount_type(tx.tx())?;
                if tx_type == deposit {
                    let c = self.get_client(tx.client())?;
                    c.chargeback(amount)?;
                }
            },
        };
        
        Ok(())
    }

    fn get_tx_amount_type(&self, id: &u32) -> Result<(f64, TransactionType)> {
        let tx = self.get_transaction(id)?;
        match tx.amount(){
            Some(a) => Ok((a, *tx.tx_type())),
            None => Err(Box::new(LedgerError::MissingTransactionAmount(*id)))
        }
    }

    //Create a default client if one does not yet exist
    fn init_client(&mut self, id: &u16){
        if  self.client_table.contains_key(id) == false {
            self.client_table.insert(*id, Client::from_id(*id));
        }
    }

    fn get_client(&mut self, id: &u16) -> Result<&mut Client>{
        match self.client_table.get_mut(id){
            Some(c) => Ok(c),
            None => Err(Box::new(LedgerError::MissingClient(*id)))
        }
    }

    fn get_transaction(&self, id: &u32) -> Result<&Transaction>{
        match self.transaction_table.get(id){
            Some(c) => Ok(c),
            None => Err(Box::new(LedgerError::MissingTransaction(*id)))
        }
    }

    pub fn write_output(self){
        let mut wtr = csv::Writer::from_writer(io::stdout());
        for (_id, client) in self.client_table {
            wtr.serialize(client).unwrap();
        }
        wtr.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {

   use super::*;
    #[test]
    fn test_apply_transactions() {
        let mut ledger : Ledger = Default::default();

        let txs = vec![
            Transaction::from(deposit, 1, 1, Some(10.0)),
            Transaction::from(deposit, 1, 2, Some(5.0)),
            Transaction::from(withdrawal, 1, 3, Some(1.0)),
            Transaction::from(dispute, 1, 2, None),
            Transaction::from(resolve, 1, 2, None),
            Transaction::from(withdrawal, 1, 3, Some(1.0)),
            Transaction::from(dispute, 1, 2, None),
            Transaction::from(chargeback, 1, 2, None),
            Transaction::from(deposit, 2, 4, Some(10.0)),
            Transaction::from(withdrawal, 2, 5, Some(5.0)),
            Transaction::from(deposit, 2, 6, Some(5.0)),
            Transaction::from(dispute, 2, 6, None),
            Transaction::from(dispute,1,999,None),
            ];

        for tx in txs {
            ledger.apply_transaction(&tx).unwrap();
        }

        let clients_expected = vec![
            Client::from(1, 8.0, 0.0, true),
            Client::from(2, 5.0, 5.0, false)
        ];

        assert_eq!(clients_expected.len(), ledger.client_table.len());

        for expected in clients_expected {
            let actual = ledger.client_table.get(expected.id()).unwrap();
            assert_eq!(expected, *actual);
        }

        ledger.write_output();

    }
}