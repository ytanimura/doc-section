# Fruit

## overview

A module defining fruit types.

## banana

A banana characterized by its peel thickness.

## banana_thickness

The thickness of the banana's peel.

## orange

An orange characterized by its sugar content.

## orange_sugar_content

The orange's sugar content.

## peeled_banana

A peeled banana.

## banana::new

Creates a banana with the given peel thickness.

## banana::peeling

Peels the banana.

### Examples

```rust
use fruit::*;

let banana = Banana::new(3.0);
assert_eq!(banana.thickness, 3.0);
assert_eq!(banana.peeling(), PeeledBanana);
```
