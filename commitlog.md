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
| timestamp     | data coming     | 128           |
| op type       | ins/del/lock    | 4            |
| key length    | < 1mb           | 32           | 
| value length  | < 1gb           | 32           |  
| key bytes     | ~               | ~            |
| value bytes   | ~               | ~            |
                  
The length of header is 128b + 4b + 32b + 32b ~ 196b 

##### Commitlog.index
A group of values in binary format
The maximum length is 1000_000_000 + 1000_000 + 17 ~ 4b
