static INDEX_FILE_NAME: &str = "commit_log.idx";

struct IndexFile {
    path: String,
}

pub struct Index {
    val: u32
}

impl Index {
    fn from(bytes: &[u8;4]) -> Index {
        let val =  u32::from_be_bytes(*bytes);
        Index { val }
    }

    fn to(&self) -> [u8; 4] {
        self.val.to_be_bytes()
    }
}

impl IndexFile {
    pub fn read(&self) -> Vec<Index> {
        vec![]
    }
}


