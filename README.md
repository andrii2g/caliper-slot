# caliper-slot

A dependency-free Rust library and SVG CLI for the minimum width of a convex
polygon. The width is the distance between the closest pair of parallel lines
that enclose and touch the polygon.

## Run the CLI

```sh
cargo run -- examples/rotated-rectangle.txt target/width.svg
```

The input has one `x y` pair per line. List at least three finite vertices in
clockwise or counterclockwise boundary order and omit the repeated closing
vertex. Supply a strictly convex polygon. The library checks basic input
errors and assumes convexity and boundary order.

The example is a 30 × 10 rectangle rotated by 30°. Its width is 10, subject
to floating-point rounding. The SVG shows both support lines, highlights the
reference edge and opposite vertex, and draws the perpendicular gap.

## Use the library

```rust
use caliper_slot::{Point, minimum_width};

let polygon = [
    Point { x: 0.0, y: 0.0 },
    Point { x: 30.0, y: 0.0 },
    Point { x: 30.0, y: 10.0 },
    Point { x: 0.0, y: 10.0 },
];
let witness = minimum_width(&polygon).expect("valid rectangle");
assert_eq!(witness.width, 10.0);
// `edge_start` and `opposite_vertex` are indices into `polygon`.
```

## Why the contact only moves forward

Some minimum-width strip has a support line containing a polygon edge. For
each edge, the sweep finds the farthest vertex by projecting onto the edge's
inward normal. As the edge normals turn in boundary order, the farthest vertex
also advances in boundary order. The sweep starts from the previous contact
and follows the boundary until the projection would decrease. Equal
projections on a parallel opposite edge select its forward endpoint.
[Houle and Toussaint, *Computing the Width of a Set*, §2](https://www-cgrl.cs.mcgill.ca/~godfried/publications/width.pdf)

An initial scan seeds the contact. After that, advancing it costs at most one
circuit of the polygon. The sweep uses O(n) time and O(1) auxiliary space.
An exhaustive scan makes n² projections because it starts over at every edge.

```sh
cargo test
cargo test rotating_calipers_match_exhaustive_projections -- --nocapture
```

The comparison checks each edge against an independent projection scan, tests
both windings and every starting vertex, and prints the contact advance and
exhaustive projection counts for each example.
