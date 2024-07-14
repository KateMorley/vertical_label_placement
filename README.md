# Vertical label placement

Functions for vertical label placement that minimise the maximum absolute offset of any label from its preferred position, while respecting limits on how high or low labels may be placed. This crate serves as a reference implementation of the algorithm described in Kate Morley’s article [Vertical label placement](https://iamkate.com/code/vertical-label-placement/).

# Examples

Placing labels, respecting a minimum separation:

```rust
let preferred_positions = vec![-10, -1, 1, 10];

let permitted_positions = vertical_label_placement::place(&preferred_positions, 10);

assert_eq!([-15, -5, 5, 15], *permitted_positions);
```

Placing labels, respecting a minimum separation and minimum and maximum positions:

```rust
let preferred_positions = vec![-10, -1, 1, 10];

let permitted_positions = vertical_label_placement::place_with_limits(
    &preferred_positions,
    10,
    0,
    100
);

assert_eq!([0, 10, 20, 30], *permitted_positions);
```
