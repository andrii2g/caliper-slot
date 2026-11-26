use super::{GeometryError, Point, minimum_width, sweep};

const RECTANGLE: &str = include_str!("../examples/rotated-rectangle.txt");

#[test]
fn rotated_rectangle_is_ten_units_wide() {
    let vertices = fixture(RECTANGLE);
    let witness = minimum_width(&vertices).unwrap();
    close(witness.width, 10.0);
    check_contact(&vertices, witness.edge_start, witness.opposite_vertex, 10.0);
}

#[test]
fn rotating_calipers_match_exhaustive_projections() {
    for (name, source) in [
        ("rotated rectangle", RECTANGLE),
        ("triangle", include_str!("../examples/triangle.txt")),
        (
            "skewed pentagon",
            include_str!("../examples/skewed-pentagon.txt"),
        ),
        (
            "parallel edges",
            include_str!("../examples/parallel-edges.txt"),
        ),
    ] {
        let mut vertices = fixture(source);
        let count = vertices.len();
        for reversed in [false, true] {
            if reversed {
                vertices.reverse();
            }
            for shift in 0..count {
                let mut candidates = Vec::new();
                let (best, advances) =
                    sweep(&vertices, |candidate| candidates.push(candidate)).unwrap();
                let (expected, projections) = exhaustive_widths(&vertices);

                assert_eq!(candidates.len(), count);
                for (edge, (candidate, expected_width)) in
                    candidates.iter().zip(&expected).enumerate()
                {
                    assert_eq!(candidate.edge_start, edge);
                    close(candidate.width, *expected_width);
                    check_contact(&vertices, edge, candidate.opposite_vertex, *expected_width);
                }
                close(
                    best.width,
                    expected.into_iter().fold(f64::INFINITY, f64::min),
                );
                assert!(advances <= count);
                assert_eq!(projections, count * count);
                if !reversed && shift == 0 {
                    println!(
                        "{name}: {advances} contact advances after seeding, {projections} exhaustive projections"
                    );
                }
                vertices.rotate_left(1);
            }
        }
    }
}

#[test]
fn chooses_forward_endpoint_of_parallel_support_edge() {
    let vertices = fixture("0 0\n30 0\n30 10\n0 10");
    let witness = minimum_width(&vertices).unwrap();
    assert_eq!(witness.edge_start, 0);
    assert_eq!(witness.opposite_vertex, 3);
}

#[test]
fn rejects_basic_input_errors() {
    assert_eq!(minimum_width(&[]), Err(GeometryError::TooFewVertices));
    assert_eq!(
        minimum_width(&fixture("0 0\n1 0\n0 NaN")),
        Err(GeometryError::NonFiniteCoordinate)
    );
    assert_eq!(
        minimum_width(&fixture("0 0\n1 0\n0 1\n0 0")),
        Err(GeometryError::ZeroLengthEdge)
    );
    assert_eq!(
        minimum_width(&fixture("0 0\n1 0\n2 0")),
        Err(GeometryError::CollinearFirstTurn)
    );
}

fn fixture(source: &str) -> Vec<Point> {
    source
        .lines()
        .map(|line| {
            let (x, y) = line.split_once(' ').unwrap();
            Point {
                x: x.parse().unwrap(),
                y: y.parse().unwrap(),
            }
        })
        .collect()
}

fn exhaustive_widths(vertices: &[Point]) -> (Vec<f64>, usize) {
    let mut projections = 0;
    let widths = (0..vertices.len())
        .map(|edge| {
            let normal = unit_normal(vertices, edge);
            let mut low = f64::INFINITY;
            let mut high = f64::NEG_INFINITY;
            for point in vertices {
                let value = normal.x * point.x + normal.y * point.y;
                low = low.min(value);
                high = high.max(value);
                projections += 1;
            }
            high - low
        })
        .collect();
    (widths, projections)
}

fn check_contact(vertices: &[Point], edge: usize, opposite: usize, width: f64) {
    let normal = unit_normal(vertices, edge);
    let project = |p: Point| normal.x * p.x + normal.y * p.y;
    let reference = project(vertices[edge]);
    let contact = project(vertices[opposite]);
    close((contact - reference).abs(), width);
    for &point in vertices {
        let value = project(point);
        assert!(value >= reference.min(contact) - 1e-12);
        assert!(value <= reference.max(contact) + 1e-12);
    }
}

fn unit_normal(vertices: &[Point], edge: usize) -> Point {
    let a = vertices[edge];
    let b = vertices[(edge + 1) % vertices.len()];
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length = dx.hypot(dy);
    Point {
        x: -dy / length,
        y: dx / length,
    }
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "expected {expected}, got {actual}"
    );
}
