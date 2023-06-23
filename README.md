# Google AIP Ordering Parser

https://google.aip.dev/132

```rust
let order_by = OrderBy::new(" foo, bar   desc , buz.buz ")?;
println!("{}", order_by.to_string()); // output is "foo asc,bar desc,buz.buz asc"
```
