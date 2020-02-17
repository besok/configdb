use crate::store::transaction_log::FromBytes;

pub trait Reader<T> where T:FromBytes{

}