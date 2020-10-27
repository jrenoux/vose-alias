# Vose Alias
A Rust implementation of the Vose-Alias Algorithm. This algorithm allows to sample an element from a list given a discrete probability distribution. 

For a vecotr of `n` elements, the initialization time is `O(n)`, the sampling time is `O(1)`, and the memory usage is `O(n)`. 


## Usage
Add this to your `Cargo.toml`
```toml
[dependencies]
vose-alias = "1.0.0"
```

## Example

```rust
use vose_alias::VoseAlias

let va = VoseAlias::new(vec!["orange", "yellow", "green", "turquoise", "grey", "blue", "pink"], vec![0.125, 0.2, 0.1, 0.25, 0.1, 0.1, 0.125]);
let element = va.sample();
```

## Crate functionalities
This crate provides a `VoseAlias` structure, that stores the list of elements given by the user of the library, as well as the probability and alias tables. The probability and alias tables are created by the `new` function.

The crate also provides a `sample` function, that allows to sample an element from the element vector in constant time. The function returns the element directly. 

## External Resources
For a description of the method implemented as well as the algorithm (in pseudo-code), see [[https://www.keithschwarz.com/darts-dice-coins/]]
