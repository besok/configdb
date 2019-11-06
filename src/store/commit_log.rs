use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Error;

/// default record for index file for commit log.
/// It consists of ints(u32) meaning the length of record in commit log
#[derive(PartialEq, Debug)]
pub struct Index {
    val: u32
}

/// commit log type
#[derive(PartialEq, Debug)]
pub enum RecordType {
    Insert,
    Delete,
    Lock,
}

/// commit log record. This record saves the information before other operation for preventing data loss
/// the header consists of ts(current time), op type RecordType, key length and val length
#[derive(PartialEq, Debug)]
pub struct Record {
    timestamp: u128,
    operation: RecordType,
    key_len: u32,
    val_len: u32,
    key: Vec<u8>,
    val: Vec<u8>,
}
pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for Record{
    /// serializing op
    /// # Order
    /// - the first byte is operation see `RecordType`
    /// - then 8 bytes is timestamp
    /// - then 4 bytes is key length
    /// - then 4 bytes is val length
    /// - then key array
    /// - then val array
    fn to_bytes(&self) -> Vec<u8> {
        let op: u8 =
            match self.operation {
                RecordType::Insert => 1,
                RecordType::Delete => 2,
                RecordType::Lock => 3,
            };

        let mut bytes = vec![op];
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.key_len.to_be_bytes());
        bytes.extend_from_slice(&self.val_len.to_be_bytes());
        bytes.extend_from_slice(&self.key);
        bytes.extend_from_slice(&self.val);

        bytes
    }
}
impl ToBytes for Index{
    fn to_bytes(&self) -> Vec<u8> {
        self.val.to_be_bytes().to_vec()
    }
}

pub trait FromBytes
    where Self: Sized
{
    fn from_bytes(bytes: &[u8]) -> Result<Self, LogError>;
}

impl FromBytes for Record {
    /// deserializing op
    /// # Arguments
    /// * `bytes` - bytes array to deserialize
    ///
    /// # Order
    /// - the first byte is operation see `RecordType`
    /// - then 8 bytes is timestamp
    /// - then 4 bytes is key length
    /// - then 4 bytes is val length
    /// - then key array
    /// - then val array
    ///
    /// # Returns
    /// `Result` with Record or `LogError`
    fn from_bytes(bytes: &[u8]) -> Result<Record, LogError> {
        if bytes.is_empty() {
            return Err(LogError);
        }

        let operation: RecordType = match bytes.get(0) {
            Some(1) => RecordType::Insert,
            Some(2) => RecordType::Delete,
            Some(3) => RecordType::Lock,
            _ => panic!("the first byte should be either 1 or 2 or 3")
        };

        let timestamp = convert_128(&bytes[1..17]);
        let key_len = convert_32(&bytes[17..21]);
        let val_len = convert_32(&bytes[21..25]);
        let key = bytes[25..25 + key_len as usize].to_vec();
        let val = bytes[25 + key_len as usize..].to_vec();

        Ok(Record { timestamp, operation, key_len, val_len, key, val })
    }
}
impl FromBytes for Index {
    fn from_bytes(bytes: &[u8]) -> Result<Index, LogError> {
        let val = u32::from_be_bytes(*convert_to_fixed(bytes));
        Ok(Index { val })
    }
}


impl Record {

    /// size in bytes operation
    /// it counts size of record
    /// Generally it comes from header(16-ts,4 and 4 from key and value length , 1 op)
    /// and bytes from key and val
    pub fn size_in_bytes(&self) -> u32 {
        self.val_len + self.key_len + 16 + 4 + 4 + 1
    }

    pub fn insert_record(key: Vec<u8>, val: Vec<u8>) -> Self {
        Record::op_from(RecordType::Insert, key, val)
    }
    pub fn delete_record(key: Vec<u8>, val: Vec<u8>) -> Self {
        Record::op_from(RecordType::Delete, key, val)
    }
    pub fn lock_record(key: Vec<u8>, val: Vec<u8>) -> Self {
        Record::op_from(RecordType::Lock, key, val)
    }


    fn op_from(operation: RecordType, key: Vec<u8>, val: Vec<u8>) -> Self {
        Record {
            timestamp: time_now_millis(),
            operation,
            key_len: key.len() as u32,
            val_len: val.len() as u32,
            key,
            val,
        }
    }
}

impl Index {
    pub fn create(val: u32) -> Index {
        Index { val }
    }

    pub fn get_value(&self) -> u32 {
        self.val
    }

    pub fn array_to_bytes<T:ToBytes>(idx_array: &Vec<T>) -> Vec<u8> {
        idx_array
            .iter()
            .flat_map(|item|item.to_bytes())
            .collect()
    }

    pub fn from_bytes_array(bytes: &[u8]) -> Result<Vec<Index>, LogError> {
        Ok(
            bytes
                .chunks(4)
                .map(|ch| Index::from_bytes(ch))
                .filter(|ch| ch.is_ok())
                .map(|ch| ch.unwrap())
                .collect()
        )
    }
}

#[derive(Debug, Clone)]
pub struct LogError;

impl From<std::io::Error> for LogError {
    fn from(e: Error) -> Self {
        LogError
    }
}


fn time_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

fn convert_128(slice: &[u8]) -> u128 {
    let mut ts_array = [0; 16];
    ts_array.copy_from_slice(&slice[0..16]);
    u128::from_be_bytes(ts_array)
}

fn convert_32(slice: &[u8]) -> u32 {
    let mut ts_array = [0; 4];
    ts_array.copy_from_slice(&slice[0..4]);
    u32::from_be_bytes(ts_array)
}

fn convert_to_fixed(bytes: &[u8]) -> &[u8; 4] {
    bytes.try_into().expect("expected an array with 4 bytes")
}

#[cfg(test)]
mod tests {
    use crate::store::commit_log::{Index, Record, RecordType, FromBytes, ToBytes};

    #[test]
    fn record_test() {
        let k = [0; 10];
        let v = [0; 15];

        let rec = Record::insert_record(k.to_vec(), v.to_vec());
        assert_eq!(rec.key_len, 10);
        assert_eq!(rec.val_len, 15);
        assert_eq!(rec.key, k.to_vec());
        assert_eq!(rec.val, v.to_vec());
        assert_eq!(rec.size_in_bytes(), 50);
        assert_eq!(rec.operation, RecordType::Insert);

        let rec = Record::delete_record(k.to_vec(), v.to_vec());
        assert_eq!(rec.operation, RecordType::Delete);

        let rec = Record::lock_record(k.to_vec(), v.to_vec());
        assert_eq!(rec.operation, RecordType::Lock);

        let vec = rec.to_bytes();
        assert_eq!(vec.len(), rec.size_in_bytes() as usize);


        if let Ok(rec_from_bt) = Record::from_bytes(&vec) {
            assert_eq!(rec_from_bt, rec)
        } else {
            panic!("should be there")
        }
    }

    #[test]
    fn index_test() {
        let idx = Index { val: 1000_000_000 };

        let bts = &idx.to_bytes();
        let idx = Index::from_bytes(bts);

        assert_eq!(idx.unwrap().get_value(), 1000_000_000);

        let idx_arr = &vec![
            Index { val: 1000_000_001 },
            Index { val: 1000_000_002 },
            Index { val: 1000_000_003 }
        ];
        let arr = Index::array_to_bytes(idx_arr);
        if let Ok(res) = Index::from_bytes_array(arr.as_slice()) {
            assert_eq!(res.len(), 3);
            assert_eq!(res.contains(&Index { val: 1000_000_001 }), true);
            assert_eq!(res.contains(&Index { val: 1000_000_002 }), true);
            assert_eq!(res.contains(&Index { val: 1000_000_003 }), true);
        } else {
            panic!("assertion failed");
        }
    }
}
