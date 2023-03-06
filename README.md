`dbgen`: Database generator
===========================

[![Crates.io](https://img.shields.io/crates/v/dbgen.svg)](https://crates.io/crates/dbgen)
[![Build status](https://github.com/kennytm/dbgen/workflows/Rust/badge.svg)](https://github.com/kennytm/dbgen/actions?query=workflow%3ARust)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.txt)

`dbgen` is a program to quickly generate random SQL dump of a table following a given set of
expressions.

* Usage
    * [Download and install](Download.md)
    * [Table generator `dbgen`](CLI.md)
    * [Schema generator `dbschemagen`](SchemaGen.md)

* Reference
    * [Template reference](Template.md)
    * [Advanced template features](TemplateAdvanced.md)

* Database generator `dbdbgen`
    * [`dbdbgen` tutorial](DbdbgenTutorial.md)
    * [`dbdbgen` reference](Dbdbgen.md)

* [WASM playground](https://kennytm.github.io/dbgen/)

## Serve generated data over S3
1. Prepare template
```sql
CREATE TABLE `test`.`t` (
  `col1` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  /*{{ rownum }}*/
  `col2` bigint(20) unsigned NOT NULL,
  /*{{ rand.range(1, 9223372036854775808) }}*/
  `col3` bigint(20) unsigned NOT NULL  ,
  /*{{ rand.range(1, 9223372036854775808) }}*/
  `col4` bigint(20) unsigned NOT NULL  ,
  /*{{ rand.range(1, 9223372036854775808) }}*/
  `col5` bigint(20) unsigned NOT NULL  ,
  /*{{ rownum }}*/
  `col6` tinyint(3) unsigned NOT NULL  ,
  /*{{ rand.range(1, 10) }}*/
  `col7` varchar(10) NOT NULL  ,
  /*{{ rand.regex('[a-z0-9]+', '', 5) }}*/
  `col8` decimal(36,18) NOT NULL DEFAULT '0.000000000000000000'  ,
  /*{{ rand.range(1, 100000) }}*/
  `col8` decimal(36,18) NOT NULL DEFAULT '0.000000000000000000'  ,
  /*{{ rand.range(1, 10000000) }}*/
  `col9` tinyint(4) NOT NULL DEFAULT '0'  ,
  /*{{ rand.range(0, 20) }}*/
  `col10` tinyint(4) NOT NULL  ,
  /*{{ rand.range(0, 5) }}*/
  `col11` bigint(20) unsigned NOT NULL DEFAULT '0'  ,
  /*{{ rand.range(1610535068, 1676882755) }}*/
  `col12` bigint(20) unsigned NOT NULL DEFAULT '0'  ,
  /*{{ rand.range(1610535068, 1676882755) }}*/
  `col13` bigint(20) unsigned NOT NULL DEFAULT '0'  ,
  /*{{ rand.range(1, 10000) }}*/
  PRIMARY KEY (`f_id`)
);
```
2. Run dbgen
```bash
> dbgen -i template.sql \
  -N 4000000 \
  -R 2000000 \
  -r 1000  \
  --s3 \
  --s3-bucket=test \
  --s3-access-key=admin \
  --s3-secret-key=admin

Using seed: fde578cb911094bde1f0a4a117e3a26c49d0ad4ea52cc91708a1068e75e8ebab
Done!                                                                                                                                                                                                         
Size     552.27 MB / 552.27 MB 🕖  220.86 MB/s                                                                                                                                                              s 
S3 server is running at http://127.0.0.1:9000/
```

3. Access file using rclone

List bucket
```
> rclone lsd s3://
          -1 2023-03-06 16:36:34        -1 test
```
List objects
```
> rclone ls s3://test
      737 test.t-schema.sql
288438914 test.t.1.sql
290655874 test.t.2.sql
```
Download objects
```
> rclone -P copy s3://test test                                                16:38:14
Transferred:      552.269 MiB / 552.269 MiB, 100%, 174.229 MiB/s, ETA 0s
Transferred:            3 / 3, 100%
Elapsed time:         3.3s
> ls test                                                                   3s 16:38:51
test.t.1.sql  test.t.2.sql  test.t-schema.sql
```

