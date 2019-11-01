use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};

static INDEX_FILE_NAME: &str = "commit_log.idx";

struct IndexFile {
    path: String,
}

/// default record for index file for commit log.
/// It consists of ints(u32) meaning the length of record in commit log
pub struct Index {
    val: u32
}

pub enum RecordType {
    Insert,
    Delete,
    Lock,
}

pub struct Record {
    timestamp: u128,
    operation: RecordType,
    key_len: u32,
    val_len: u32,
    key: Box<[u8]>,
    val: Box<[u8]>,
}

fn time_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_millis()
}

fn convert_128(slice: &[u8]) -> u128 {
    let mut ts_array = [0; 16];
    ts_array.copy_from_slice(&slice[0..17]);
    u128::from_be_bytes(ts_array)
}
fn convert_32(slice: &[u8]) -> u32 {
    let mut ts_array = [0; 4];
    ts_array.copy_from_slice(&slice[0..5]);
    u32::from_be_bytes(ts_array)
}

impl Record {
    pub fn from_bytes(bytes: &Vec<u8>) -> Result<Record, LogError> {
        if bytes.is_empty() {
            return Err(LogError("bytes should not be empty"));
        }

        let operation: RecordType = match bytes.get(0) {
            Some(1) => RecordType::Insert,
            Some(2) => RecordType::Delete,
            Some(3) => RecordType::Lock,
            _ => panic!("the first byte should be either 1 or 2 or 3")
        };

        let timestamp = convert_128(&bytes[1..17]);

        Ok(Record {
            timestamp,
            operation,
            key_len: 0,
            val_len: 0,
            key: Box::new([]),
            val: Box::new([]),
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
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

    pub fn size_in_bytes(&self) -> u32 {
        self.val_len + self.key_len + 16 + 4 + 4 + 1
    }

    pub fn insert_record(key: Box<[u8]>, val: Box<[u8]>) -> Self {
        Record::op_from(RecordType::Insert, key, val)
    }
    pub fn delete_record(key: Box<[u8]>, val: Box<[u8]>) -> Self {
        Record::op_from(RecordType::Delete, key, val)
    }
    pub fn lock_record(key: Box<[u8]>, val: Box<[u8]>) -> Self {
        Record::op_from(RecordType::Lock, key, val)
    }


    fn op_from(op: RecordType, key: Box<[u8]>, val: Box<[u8]>) -> Self {
        Record {
            timestamp: time_now_millis(),
            operation: op,
            key_len: key.len() as u32,
            val_len: val.len() as u32,
            key,
            val,
        }
    }
}


#[derive(Debug, Clone)]
pub struct LogError(&'static str);

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }

    fn ne(&self, other: &Self) -> bool {
        self.val != other.val
    }
}


impl Index {
    fn array_to_bytes(idx_array: &Vec<Index>) -> Box<Vec<u8>> {
        let mut res: Vec<u8> = vec![];

        idx_array
            .iter()
            .for_each(|idx| res.extend(idx.to_bytes().iter()));

        return Box::new(res);
    }

    fn from_bytes_array(bytes: &[u8]) -> Result<std::vec::Vec<Index>, LogError> {
        Ok(
            bytes
                .chunks(4)
                .map(|ch| Index::from_bytes(ch))
                .collect()
        )
    }

    fn convert_to_fixed(bytes: &[u8]) -> &[u8; 4] {
        bytes.try_into().expect("expected an array with 4 bytes")
    }

    fn from_bytes(bytes: &[u8]) -> Index {
        let val = u32::from_be_bytes(*Index::convert_to_fixed(bytes));
        Index { val }
    }

    fn to_bytes(&self) -> [u8; 4] {
        self.val.to_be_bytes()
    }
}


#[cfg(test)]
mod tests {
    use crate::store::commit_log::Index;

    #[test]
    fn quick_test() {}

    #[test]
    fn index_test() {
        let idx = Index { val: 1000_000_000 };

        let bts = &idx.to_bytes();
        let idx = Index::from_bytes(bts);

        assert_eq!(idx.val, 1000_000_000);

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
