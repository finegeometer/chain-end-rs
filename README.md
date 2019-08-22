# chain-end

Manipulate the ends of a list independently.

## API

```rust
/// One end of a chain of values.
pub struct ChainEnd<T>;

impl<T> ChainEnd<T> {
    /// Create a new chain, returning the two ends.
    pub fn new(iter: impl IntoIterator<Item = T>) -> (Self, Self) {
    /// Join two chain ends.
    /// If they were two ends of the same chain, return the contents of the loop this creates.
    /// If they were ends of different chains, return None.
    pub fn connect(self, other: Self) -> Option<impl Iterator<Item = T>>;
}
```

## Usage

```rust
let (a, b) = ChainEnd::new(0..3);
let (c, d) = ChainEnd::new(3..6);

// We now have these chains:
// a - 0 - 1 - 2 - b
// c - 3 - 4 - 5 - d

// We connect b to d, creating one long chain.
assert!(b.connect(d).is_none());

// We now have this chain:
// a - 0 - 1 - 2 - 5 - 4 - 3 - c

// We connect c to a, creating a loop. The contents of the loop are returned.
assert_eq!(
    c.connect(a).unwrap().collect::<Vec<_>>(),
    vec![0, 1, 2, 5, 4, 3]
);
```

## Safety

I believe this API is safe, but the implementation isn't. Therefore, this is probably broken.