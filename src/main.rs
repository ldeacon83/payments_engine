extern crate csv;
#[macro_use]
extern crate serde_derive;

mod client;
mod ledger;
mod transaction;

use ledger::Ledger;
use std::{fs::File, env};


fn main() {
    let args: Vec<String> = env::args().collect();
    let f = File::open(&args[1]).unwrap();
    let mut rdr = csv::Reader::from_reader(f);
    

    let mut ledger: Ledger = Default::default();

    for tx in rdr.deserialize(){
        ledger.apply_transaction(&tx.unwrap()).unwrap();
    }
        
    ledger.write_output();
}


#[cfg(test)]
mod tests {

    use super::*;    

    const INPUT: &str = "type,client,tx,amount\ndeposit,1,1,10.0\nwithdrawal,1,2,2.0\ndispute,1,2,\nresolve,1,2,\nwithdrawal,1,3,2.0\ndispute,1,3,\nchargeback,1,3,\n";    

    #[test]
    fn test_io() {
        let mut ledger : Ledger = Default::default();
        let mut rdr = csv::Reader::from_reader(INPUT.as_bytes());        
        for tx in rdr.deserialize(){
            let txu = tx.unwrap();
            ledger.apply_transaction(&txu).unwrap();
        }
        ledger.write_output();
    }



}



