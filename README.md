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




