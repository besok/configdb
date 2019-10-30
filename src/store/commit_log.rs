use std::convert::TryInto;

static INDEX_FILE_NAME: &str = "commit_log.idx";

struct IndexFile {
    path: String,
}

pub struct Index {
    val: u32
}

#[derive(Debug, Clone)]
struct IndexError(&'static str);

impl Index {
    fn to_bytes_array(idx_array: &Vec<Index>) -> Box<[u8]> {
        if idx_array.is_empty() {
            return Box::new([]);
        }
        let x = idx_array
            .iter()
            .flat_map(|idx| idx.to_bytes())
            .collect();

            return Box::new([]);
    }

    fn from_bytes_array(bytes: &[u8]) -> Result<std::vec::Vec<Index>, IndexError> {
        if (bytes.len() % 4) != 0 {
            return Err(IndexError("should divide by 4"));
        }

        Ok(
            bytes
                .chunks(4)
                .map(|ch| Index::from_bytes(ch))
                .collect()
        )
    }

    fn convert(bytes: &[u8]) -> &[u8; 4] {
        bytes.try_into().expect("expected an array with 4 bytes")
    }

    fn from_bytes(bytes: &[u8]) -> Index {
        let val = u32::from_be_bytes(*Index::convert(bytes));
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
    fn index_test() {
        let idx = Index { val: 1000_000_000 };

        let bts = &idx.to_bytes();
        let idx = Index::from_bytes(bts);

        assert_eq!(idx.val, 1000_000_000);


        let result = Index::from_bytes_array(bts);
    }
}
