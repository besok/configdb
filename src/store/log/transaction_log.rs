use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Error;
use std::path::PathBuf;
use crate::store::files::*;
use std::io;
use std::fs::{File, remove_file};
use crate::store::{ToBytes, FromBytes, StoreResult, StoreError};


static LOCK_FILE: &str = "log.lock";
static IDX_FILE_NAME: &str = "log_idx.cfgdb";
static LOG_FILE_NAME: &str = "log_data.cfgdb";
static BACKUP_EXT: &str = "cfgdb.bck";


/// default struct including into itself index and log
#[derive(Debug)]
pub struct TransactionLog {
    idx: PathBuf,
    log: PathBuf,
    lock: PathBuf,
}

impl Drop for TransactionLog {
    fn drop(&mut self) {
        self.close();
    }
}

impl TransactionLog {
    pub fn close(&self) -> io::Result<()> {
        remove_file(&self.lock)
    }

    pub fn remove_files(&self) -> io::Result<()> {
        remove_file(&self.idx)?;
        remove_file(&self.log)?;
        remove_file(&self.lock)?;
        Ok(())
    }

    /// tries to create a new commit log even the file lock exists
    /// If the file lock exists tries to delete it then invoke `CommitLog::create`
    pub fn create_force(dir_str: &str) -> StoreResult<Self> {
        let mut lock = PathBuf::from(dir_str);
        lock.push(LOCK_FILE);
        if lock.exists() {
            remove_file(lock)?
        }

        TransactionLog::create(dir_str)
    }

    /// create a new commit log
    /// Create 3 files for logging.
    ///  - The first one is an index
    ///  - The second one is a log
    ///  - the third one is a lock file
    /// # Arguments
    /// * `dir` id a directory for files. If it does not exist it tries to create dirs
    ///
    /// # Examples
    ///
    /// ```rust
    /// if let Ok(c_log) = CommitLog::create(r"c:\projects\configdb\data") {}
    /// ```
    ///
    pub fn create(dir_str: &str) -> StoreResult<Self> {
        let dir = {
            let dir = PathBuf::from(dir_str);
            if dir.is_file() {
                return Err(StoreError(String::from(" err in dir.is_file() || !dir.exists()")));
            }
            if !dir.exists() {
                std::fs::create_dir_all(dir.as_path())?;
            }
            dir
        };

        Ok(TransactionLog {
            lock: {
                let mut lock = PathBuf::from(dir.clone());
                lock.push(LOCK_FILE);
                if lock.exists() {
                    return Err(StoreError(String::from(format!("lock file for {} exists", dir_str))));
                }

                File::create(lock.as_path())?;
                lock
            },
            log: {
                let mut log = PathBuf::from(dir.clone());
                log.push(LOG_FILE_NAME);
                File::create(log.as_path())?;
                log
            },
            idx: {
                let mut idx = PathBuf::from(dir.clone());
                idx.push(IDX_FILE_NAME);
                File::create(idx.as_path())?;
                idx
            },
        })
    }
    pub fn backup(&self) -> StoreResult<()> {
        let idx = &self.idx;
        let log = &self.log;
        if !idx.exists() || !log.exists() {
            return Err(StoreError(String::from(" error in !idx.exists() || !log.exists()")));
        }

        let mut idx_bk = PathBuf::from(idx);
        let mut log_bk = PathBuf::from(log);

        idx_bk.set_extension(BACKUP_EXT);
        log_bk.set_extension(BACKUP_EXT);

        copy_file(log.as_path(), log_bk.as_path())?;
        copy_file(idx.as_path(), idx_bk.as_path())
    }
    pub fn push(&self, record: &Record) -> StoreResult<usize> {
        let index = &Index::create(record.size_in_bytes());
        append_item(&self.idx, index)?;
        let r = append_item(&self.log, record)?;
        Ok(r)
    }

    /// read list of records from the end according a position
    /// # Arguments
    ///* `number_from_end` the position relative to the end. Should be more or equal 1
    /// Can return `StoreError` if number less 1
    pub fn read_all_from_end(&self, number_from_end: usize) -> StoreResult<Vec<Record>> {
        let mut r_start_pos = 0;
        let mut r_number: u64;
        let mut records: Vec<Record> = Vec::new();

        for i in 1..=number_from_end {
            let pos: u64 = i as u64 * 4;
            match read_slice_from_end::<Index>(self.idx.as_path(), pos, 4) {
                Ok(idx) => {
                    let vl = idx.get_value() as u64;
                    r_start_pos += vl;
                    r_number = vl;
                    match read_slice_from_end::<Record>(self.log.as_path(), r_start_pos, r_number) {
                        Ok(r) => records.push(r),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(records)
    }

    /// read record from the end according a position
    /// # Arguments
    ///* `number_from_end` the position relative to the end. Should be more or equal 1
    /// Can return `StoreError` if number less 1
    pub fn read_from_end(&self, pos_from_end: usize) -> StoreResult<Record> {
        let mut r_start_pos = 0;
        let mut r_number: u64 = 0;
        for i in 1..=pos_from_end {
            let pos: u64 = i as u64 * 4;
            match read_slice_from_end::<Index>(self.idx.as_path(), pos, 4) {
                Ok(idx) => {
                    let vl = idx.get_value() as u64;
                    r_start_pos += vl;
                    r_number = vl;
                }
                Err(e) => return Err(e),
            }
        }

        if r_number == 0 {
            return Err(StoreError(String::from(" error is r number == 0 ")));
        }
        read_slice_from_end::<Record>(self.log.as_path(), r_start_pos, r_number)
    }
}

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

impl ToBytes for Record {
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

impl ToBytes for Index {
    fn to_bytes(&self) -> Vec<u8> {
        self.val.to_be_bytes().to_vec()
    }
}

impl FromBytes for Record {
    /// deserializer op
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
    /// `Result` with Record or `StoreError`
    fn from_bytes(bytes: &[u8]) -> StoreResult<Record> {
        if bytes.is_empty() {
            return Err(StoreError(String::from(" bytes are empty")));
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
    fn from_bytes(bytes: &[u8]) -> StoreResult<Index> {
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

    pub fn array_to_bytes<T: ToBytes>(idx_array: &Vec<T>) -> Vec<u8> {
        idx_array
            .iter()
            .flat_map(|item| item.to_bytes())
            .collect()
    }

    pub fn from_bytes_array(bytes: &[u8]) -> StoreResult<Vec<Index>> {
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


impl From<std::io::Error> for StoreError {
    fn from(e: Error) -> Self {
        StoreError(e.to_string())
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
    use crate::store::log::transaction_log::{Index, Record, RecordType, TransactionLog, time_now_millis};
    use crate::store::{FromBytes, ToBytes};


    #[test]
    fn try_to_create_force_test() {
        if let Ok(t_log) = TransactionLog::create_force(r"test_data\force_create") {
            if let Ok(_) = TransactionLog::create(r"test_data\force_create") { panic!("") }
            if let Err(_) = TransactionLog::create_force(r"test_data\force_create") { panic!("") }
            t_log.remove_files();
        } else {
            panic!("")
        }
    }


    #[test]
    fn read_all_log_test() {
        if let Ok(t_log) = TransactionLog::create(r"test_data\read_all") {
            for i in 1..101 {
                let rec = &Record::delete_record(vec![1 as u8; i * 1], vec![1 as u8; i * 10]);
                match t_log.push(rec) {
                    Err(e) => panic!("{}", e.0),
                    _ => continue
                }
            }
            let mut sizes = vec![0; 0];
            for i in 1..101 {
                let rev_i = 101 - i;
                let expected_size = (rev_i * 1 + rev_i * 10 + 25) as u32;
                sizes.push(expected_size);
            }

            match t_log.read_all_from_end(100) {
                Ok(records) => {
                    for (i, r) in records.iter().enumerate() {
                        assert_eq!(r.size_in_bytes(), *sizes.get(i).unwrap())
                    }
                }
                Err(e) => panic!(" e {:?}", e),
            }
            t_log.remove_files();
        } else {
            panic!("panic")
        }
    }

    #[test]
    fn read_log_test() {
        if let Ok(t_log) = TransactionLog::create(r"test_data\read_partially") {
            for i in 1..101 {
                let rec = &Record::insert_record(vec![1 as u8; i * 1], vec![1 as u8; i * 10]);
                match t_log.push(rec) {
                    Err(e) => panic!("{}", e.0),
                    _ => continue
                }
            }
            for i in 1..101 {
                let rev_i = 101 - i;
                let expected_size = (rev_i * 1 + rev_i * 10 + 25) as u32;
                match t_log.read_from_end(i) {
                    Ok(r) => assert_eq!(r.size_in_bytes(), expected_size),
                    Err(e) => panic!(" e {:?}", e)
                }
            }
            t_log.remove_files();
        } else {
            panic!("panic")
        }
    }

    #[test]
    fn dummy_performance_test() {
        if let Ok(t_log) = TransactionLog::create(r"test_data\performance") {
            let start_time = time_now_millis();
            let rec = &Record::insert_record(vec![1 as u8; 10], vec![1 as u8; 100]);
            for _ in 1..1000 {
                if let Err(e) = t_log.push(rec) {
                    panic!("{}", e.0);
                }
            }
            let dur = time_now_millis() - start_time;
            println!("dur = {}", dur);
            t_log.remove_files();
        } else {
            panic!("panic")
        }
    }

    #[test]
    fn commit_log_test() {
        if let Ok(t_log) = TransactionLog::create(r"test_data\simple") {
            let rec = Record::insert_record(vec![1 as u8; 10], vec![1 as u8; 20]);

            if let Ok(size_res) = t_log.push(&rec) {
                assert_eq!(size_res, 55);
            } else {
                panic!("panic")
            }

            if let Err(e) = t_log.remove_files() {
                panic!("-> {}", e.to_string());
            }
        } else {
            panic!("panic")
        }
    }

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
