use std::convert::TryInto;

static INDEX_FILE_NAME: &str = "commit_log.idx";

struct IndexFile {
    path: String,
}
/// default record for index file for commit log.
/// It consists of ints(u32) meaning the length of record in commit log
pub struct Index {
    val: u32
}

#[derive(Debug, Clone)]
struct IndexError(&'static str);

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

    fn from_bytes_array(bytes: &[u8]) -> Result<std::vec::Vec<Index>, IndexError> {
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
    use std::borrow::Borrow;

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
