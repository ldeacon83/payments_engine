use std::{fmt, error::Error};
use serde::ser::{Serialize, Serializer, SerializeStruct};

type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError{
    InsufficientFunds,
    IncorrectSign,
}

impl Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            ClientError::InsufficientFunds => write!(f, "Insufficient funds"),
            ClientError::IncorrectSign => write!(f, "Incorrect sign"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Client {
    id: u16,
    available: f64,
    held: f64,
    locked: bool
}

impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where 
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Client", 5)?;
        state.serialize_field("client", &self.id)?;
        state.serialize_field("available", &format!("{:.4}", self.available))?;
        state.serialize_field("held", &format!("{:.4}", self.held))?;
        state.serialize_field("total", &format!("{:.4}", self.total()))?;
        state.serialize_field("locked", &self.locked)?;
        state.end()
    }
}



impl Client {
    #[allow(dead_code)]
    pub fn from(id: u16, available: f64, held: f64, locked: bool) -> Self{
        Client{id, available, held, locked}
    }

    pub fn from_id(id: u16) -> Self {
        Client{id, ..Default::default()}
    }

    //Use negative amount for withdraw
    pub fn deposit(&mut self, amount: f64) -> Result<()> {
        if amount.is_sign_negative() && amount.abs() > self.available { return Err(ClientError::InsufficientFunds) }; 
        self.available = self.available + amount;
        Ok(())
    }
        
    //Use negative amount for release
    pub fn hold(&mut self, amount: f64) -> Result<()>{
        if amount.is_sign_negative() {
            if amount.abs() > self.held { return Err(ClientError::InsufficientFunds); }        
        } else if amount > self.available {
            return Err(ClientError::InsufficientFunds.into());
        }
        self.held = self.held + amount;
        self.available = self.available - amount;
        Ok(())
    }

    pub fn chargeback(&mut self, amount: f64) -> Result<()> {
        if amount.is_sign_negative() {
            return Err(ClientError::IncorrectSign);       
        }
        if amount > self.held {
            return Err(ClientError::InsufficientFunds)
        }
        if amount.abs() > self.held { return Err(ClientError::InsufficientFunds); } 
        self.held = self.held - amount;       
        self.locked = true;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_locked(&mut self, locked: bool){
        self.locked = locked.to_owned();
    }

    #[allow(dead_code)]
    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn total(&self) -> f64 {
        self.available + self.held
    }

    #[allow(dead_code)]
    pub fn available(&self) -> f64 {
        self.available
    }

    #[allow(dead_code)]
    pub fn held(&self) -> f64 {
        self.held
    }

    #[allow(dead_code)]
    pub fn id(&self) -> &u16 {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    const AVAILABLE: f64 = 100.1221;
    const HELD: f64 = 2345.5443;
    const LOCKED: bool = false;

    fn get_test_client() -> Client {
        Client {
            id: 1,
            available: AVAILABLE,
            held: HELD,
            locked: LOCKED,
        }
    }

    #[test]
    fn test_output() {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.serialize(get_test_client()).unwrap();
        wtr.flush().unwrap();
    }

    #[test]
    fn test_deposit() {
        let mut client = get_test_client();
        assert_eq!(client.total(), AVAILABLE + HELD);
        let deposit = 1345.678;
        client.deposit(deposit).unwrap();
        assert_eq!(client.total(), AVAILABLE + HELD + deposit);
        let this_available = client.available();
        client.deposit(-this_available).unwrap();
        assert_eq!(client.available(), 0.0);
        assert_eq!(client.deposit(-1.0), Err(ClientError::InsufficientFunds));
    }

    #[test]
    fn test_hold() {
        let mut client = get_test_client();
        assert_eq!(client.total(), AVAILABLE + HELD);
        let hold = 50.21;
        client.hold(hold).unwrap();
        assert_eq!(client.total(), AVAILABLE + HELD);
        let this_available = client.available();
        client.hold(this_available).unwrap();
        assert_eq!(client.available(), 0.0);
        assert_eq!(client.total(), AVAILABLE + HELD);
        assert_eq!(client.hold(1.0), Err(ClientError::InsufficientFunds));
        client.hold(-client.total()).unwrap();
        assert_eq!(client.available(), AVAILABLE + HELD);
        assert_eq!(client.held(), 0.0);
        assert_eq!(client.hold(-1.0), Err(ClientError::InsufficientFunds));
    }

    #[test]
    fn test_lock() {
        let mut client = get_test_client();
        assert_eq!(client.locked(), LOCKED);
        client.set_locked(true);
        assert_eq!(client.locked(), true);
    }

    #[test]
    fn test_amounts() {
        let client = get_test_client();
        assert_eq!(client.locked(), LOCKED);
        assert_eq!(client.available(), AVAILABLE);
        assert_eq!(client.held(), HELD);
        assert_eq!(client.total(), HELD + AVAILABLE);
    }

}