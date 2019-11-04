use std::path::Path;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, Error, ErrorKind};
use std::io;

static INDEX_FILE_NAME: &str = "commit_log.idx";


fn read_slice_bytes_internal(from: u64, to: u64, file_size: u64, f: File) -> Result<Vec<u8>, Error> {
    if from >= file_size || to > file_size || from >= to {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let mut res: Vec<u8> = vec![];
    for (i, b_res) in f.bytes().into_iter().enumerate() {
        if i >= from as usize && i < to as usize {
            match b_res {
                Ok(b) => res.push(b),
                Err(err) => return Err(err),
            }
        }
        if i>= to as usize{
            break;
        }
    };
    Ok(res)
}

fn read_slice_bytes(p: &Path, from: u64, number: u64) -> io::Result<Vec<u8>> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let to = from + number;
    read_slice_bytes_internal(from, to, file_size, f)
}

fn read_slice_from_end_bytes(p :&Path, from:u64,number:u64) -> io::Result<Vec<u8>>{
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let start_pos = file_size - from;
    let fin_pos = start_pos + number;
    read_slice_bytes_internal(start_pos, fin_pos, file_size, f)
}

fn read_from_end_bytes(p: &Path, number: u64) -> io::Result<Vec<u8>> {
    let f = File::open(p)?;
    let file_size = f.metadata()?.len();
    let start_pos = file_size - number;
    read_slice_bytes_internal(start_pos, file_size, file_size, f)
}

fn append_bytes(p: &Path, bytes: &[u8]) -> io::Result<usize> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(p)?
        .write(bytes)
}

#[cfg(test)]
mod tests {
    use crate::store::store::{append_bytes, read_from_end_bytes, read_slice_bytes, read_slice_from_end_bytes};
    use std::path::Path;
    use crate::store::commit_log::Index;
    use std::fs::File;

    #[test]
    fn simple_test() {
        let file = File::create(Path::new("test.data")).unwrap();
        let idx = Index::create(1111);

        append_bytes(Path::new("test.data"), &Index::create(1).to_bytes());
        append_bytes(Path::new("test.data"), &Index::create(2).to_bytes());
        append_bytes(Path::new("test.data"), &Index::create(3).to_bytes());
        append_bytes(Path::new("test.data"), &Index::create(4).to_bytes());
        append_bytes(Path::new("test.data"), &Index::create(5).to_bytes());


        if let Ok(bytes) = read_from_end_bytes(Path::new("test.data"), 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(5))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_bytes(Path::new("test.data"), 0, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(1))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_bytes(Path::new("test.data"), 4, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(2))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_bytes(Path::new("test.data"), 8, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(3))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_bytes(Path::new("test.data"), 12, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(4))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_bytes(Path::new("test.data"), 16, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(5))
        } else {
            panic!("panic")
        }
        if let Ok(bytes) = read_slice_from_end_bytes(Path::new("test.data"), 8, 4) {
            let idx = Index::from_bytes(bytes.as_slice());
            assert_eq!(idx, Index::create(4))
        } else {
            panic!("panic")
        }


    }
}