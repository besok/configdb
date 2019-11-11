use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, BufReader};
use std::{io, fs};
use crate::store::commit_log::{LogError, FromBytes, ToBytes};


pub fn append_item<T: ToBytes>(p: &Path, item: &T) -> io::Result<usize> {
    append_bytes(p, item.to_bytes().as_slice())
}

pub fn copy_file(src: &Path, dst: &Path) -> Result<(), LogError> {
    fs::copy(src, dst)?;
    Ok(())
}

pub fn read_slice<T: FromBytes>(p: &Path, from: u64, number: u64) -> Result<T, LogError> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let to = from + number;
    read_slice_bytes_internally(from, to, file_size, f)
        .and_then(|bs| FromBytes::from_bytes(bs.as_slice()))
}

pub fn read_from_end<T: FromBytes>(p: &Path, number: u64) -> Result<T, LogError> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let start_pos = file_size - number;
    read_slice_bytes_internally(start_pos, file_size, file_size, f)
        .and_then(|bs| FromBytes::from_bytes(bs.as_slice()))
}

pub fn read_slice_from_end<T: FromBytes>(p: &Path, from: u64, number: u64) -> Result<T, LogError> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let start_pos = file_size - from;
    let fin_pos = start_pos + number;
    read_slice_bytes_internally(start_pos, fin_pos, file_size, f)
        .and_then(|bs| FromBytes::from_bytes(bs.as_slice()))
}


pub fn read_all_file_bytes(p: &Path) -> Result<Vec<u8>, LogError> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    read_slice_bytes_internally(0, file_size, file_size, f)
}

fn append_bytes(p: &Path, bytes: &[u8]) -> io::Result<usize> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(p)?
        .write(bytes)
}

fn read_slice_bytes_internally(from: u64, to: u64, file_size: u64, f: File) -> Result<Vec<u8>, LogError> {
    if from >= file_size || to > file_size || from >= to {
        return Err(
            LogError(String::from(
                format!("from:{f} >= file_size:{fs} || to:{t} > file_size:{fs} || from:{f} >= to:{t}",
                        f = from, fs = file_size, t = to)))
        );
    }

    let range = (to - from) as usize;
    let vec: Vec<u8> =
        BufReader::with_capacity( 1024 , f)
            .bytes()
            .skip(from as usize)
            .take(range)
            .filter_map(Result::ok)
            .collect();

    if vec.len() == range {
        Ok(vec)
    } else {
        Err(LogError(String::from("some of bytes are broken")))
    }
}


#[cfg(test)]
mod tests {
    use crate::store::files::{read_from_end, read_slice, read_slice_from_end, read_all_file_bytes, append_item};
    use std::path::Path;
    use crate::store::commit_log::{Index, Record};
    use std::fs::{File, remove_file};

    #[test]
    fn simple_test() {
        let p = Path::new("test.data");
        let _ = File::create(p).unwrap();

        append_item(p, &Index::create(1));
        append_item(p, &Index::create(2));
        append_item(p, &Index::create(3));
        append_item(p, &Index::create(4));
        append_item(p, &Index::create(5));


        if let Ok(idx) = read_from_end::<Index>(p, 4) {
            assert_eq!(idx, Index::create(5))
        } else {
            panic!("panic")
        }
        if let Ok(idx) = read_slice::<Index>(p, 0, 4) {
            assert_eq!(idx, Index::create(1))
        } else {
            panic!("panic")
        }
        if let Ok(idx) = read_slice::<Index>(p, 4, 4) {
            assert_eq!(idx, Index::create(2))
        } else {
            panic!("panic")
        }
        if let Ok(idx) = read_slice::<Index>(p, 8, 4) {
            assert_eq!(idx, Index::create(3))
        } else {
            panic!("panic")
        }
        if let Ok(idx) = read_slice::<Index>(p, 12, 4) {
            assert_eq!(idx, Index::create(4))
        } else {
            panic!("panic")
        }
        if let Ok(idx) = read_slice::<Index>(p, 16, 4) {
            assert_eq!(idx, Index::create(5))
        } else {
            panic!("panic")
        }

        match read_slice_from_end::<Index>(p, 8, 4) {
            Ok(idx) => assert_eq!(idx, Index::create(4)),
            Err(_) => panic!("panic"),
        }

        let _ = remove_file(p);
    }

    #[test]
    fn normal_test() {
        let idx_file = Path::new("index.data");
        let log_file = Path::new("log.data");

        let _ = File::create(idx_file).unwrap();
        let _ = File::create(log_file).unwrap();


        let insert_rec = Record::insert_record(vec![1, 1, 1], vec![2, 2, 2]);
        let delete_rec = Record::delete_record(vec![1, 1, 1, 1], vec![2, 2, 2, 1]);
        let lock_rec = Record::lock_record(vec![1, 1], vec![2]);

        append_item(idx_file, &Index::create(insert_rec.size_in_bytes()));
        append_item(idx_file, &Index::create(delete_rec.size_in_bytes()));
        append_item(idx_file, &Index::create(lock_rec.size_in_bytes()));

        append_item(log_file, &insert_rec);
        append_item(log_file, &delete_rec);
        append_item(log_file, &lock_rec);

        if let Ok(bt) = read_all_file_bytes(idx_file) {
            if let Ok(idx_vec) = Index::from_bytes_array(bt.as_slice()) {
                let mut str_pos = 0;
                let val = idx_vec.get(0).unwrap().get_value() as u64;

                match read_slice::<Record>(log_file, str_pos, val) {
                    Ok(rec) => {
                        assert_eq!(rec, insert_rec);
                        str_pos += val;
                    }
                    _ => panic!("panic")
                }

                let val = idx_vec.get(1).unwrap().get_value() as u64;
                match read_slice::<Record>(log_file, str_pos, val) {
                    Ok(rec) => {
                        assert_eq!(rec, delete_rec);
                        str_pos += val;
                    }
                    _ => panic!("panic")
                }
                let val = idx_vec.get(2).unwrap().get_value() as u64;
                match read_slice::<Record>(log_file, str_pos, val) {
                    Ok(rec) => {
                        assert_eq!(rec, lock_rec);
                    }
                    _ => panic!("panic")
                }
            } else {
                panic!("panic")
            }
        } else {
            panic!("panic")
        }


        let _ = remove_file(idx_file);
        let _ = remove_file(log_file);
    }
}