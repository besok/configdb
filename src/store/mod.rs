pub mod log;
pub mod files;
pub mod memory;
pub mod disk;
pub mod structures;

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

pub type StoreResult<K> = Result<K, StoreError>;
#[derive(Debug, Clone)]
pub struct StoreError(pub String);



pub trait FromBytes where Self: Sized {
    fn from_bytes(bytes: &[u8]) -> StoreResult<Self>;
}



#[cfg(test)]
mod tests{
    #[test]
    fn test(){}

}





