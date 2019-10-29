#### Commit log
It consists of commitlog.data and commitlog.index
The first one is commit log and the second one has only lengths of records

##### Commitlog.data
A group of records in binary format

###### Structure of record
| timestamp | op type(1 bit) | key length(less 1000 000 ~ 1mb) | value length (less 1000000000 ~ 1 gb) | key bytes | value bytes|   

The length of header is 8b + 1b + 4b + 4b ~ 17b

##### Commitlog.index
A group of values in binary format
The maximum length is 1000_000_000 + 1000_000 + 17 ~ 4b
