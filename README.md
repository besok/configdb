### cfgdb

distributed kv database to store configuration files taking major concepts from cassandra,rocks db, leveldb and etc
It admits to creating a lock on key thereby not to allow read/update/delete record.

The major structure is lsm tree :
- memory is skiplist
- cuckoo filter to answer that an element absences
- transaction logs - simple file/byte log.


#### todos
- cuckoo filter
    - fingerprint too slow
    - add remove op
    - add rehash op

#### Commit log
It consists of commitlog.data and commitlog.index
- commitlog.data is commit log
- commitlog.index as only lengths of records from previous one


##### Commitlog.data
A group of records in binary format

###### Structure of record
- timestamp
- operation type
- key length
- value length
- key bytes
- value bytes


| Timestamp     | description     | size in bits |
| :------------ |:---------------:| ------------:|
| timestamp     | data coming     | 128          |
| op type       | ins/del/lock    | 1            |
| key length    | < 1mb           | 32           |
| value length  | < 1gb           | 32           |
| key bytes     | ~               | ~            |
| value bytes   | ~               | ~            |

The length of header is 128b(16) + 1b + 32b(4) + 32b(4) = 25 bytes

##### Commitlog.index
A group of values in binary format
The maximum length is 1000_000_000 + 1000_000 + 17~ 4b



