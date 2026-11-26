//! Minimum width of an ordered, strictly convex polygon.

use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidthWitness {
    pub width: f64,
    /// Index of the first endpoint of the reference edge. The second wraps around.
    pub edge_start: usize,
    /// Index of a vertex on the opposite support line.
    pub opposite_vertex: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    TooFewVertices,
    NonFiniteCoordinate,
    ZeroLengthEdge,
    CollinearFirstTurn,
    NumericalRange,
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooFewVertices => "expected at least three vertices",
            Self::NonFiniteCoordinate => "vertex coordinates must be finite",
            Self::ZeroLengthEdge => "consecutive vertices must be distinct",
            Self::CollinearFirstTurn => "the first three vertices must form a strict turn",
            Self::NumericalRange => {
                "the support sweep could not resolve the polygon in f64 precision"
            }
        })
    }
}

impl Error for GeometryError {}

/// Finds the minimum distance between parallel support lines in O(n) time
/// and O(1) auxiliary space.
///
/// Supply the boundary vertices of a strictly convex polygon in clockwise
/// or counterclockwise order, without repeating the first vertex at the end.
/// The function checks basic input errors and assumes boundary order and
/// strict convexity. It uses ordinary floating-point arithmetic.
pub fn minimum_width(vertices: &[Point]) -> Result<WidthWitness, GeometryError> {
    sweep(vertices, |_| {}).map(|(witness, _)| witness)
}

fn sweep(
    vertices: &[Point],
    mut visit: impl FnMut(WidthWitness),
) -> Result<(WidthWitness, usize), GeometryError> {
    let count = vertices.len();
    if count < 3 {
        return Err(GeometryError::TooFewVertices);
    }
    if vertices
        .iter()
        .any(|p| !p.x.is_finite() || !p.y.is_finite())
    {
        return Err(GeometryError::NonFiniteCoordinate);
    }
    let next = |index| (index + 1) % count;
    let first = direction(vertices[0], vertices[1])?;
    let second = direction(vertices[1], vertices[2])?;
    let turn = cross(first, second);
    if turn == 0.0 {
        return Err(GeometryError::CollinearFirstTurn);
    }
    let winding = turn.signum();

    // Seed the opposing contact once with a full scan for the first edge.
    let mut contact = 0;
    let mut first_width = 0.0;
    for (index, &point) in vertices.iter().enumerate().skip(1) {
        let height = projection(first, vertices[0], point, winding)?;
        if height > first_width {
            first_width = height;
            contact = index;
        }
    }

    let mut best = WidthWitness {
        width: f64::INFINITY,
        edge_start: 0,
        opposite_vertex: contact,
    };
    let mut advances = 0;
    for (edge_start, &anchor) in vertices.iter().enumerate() {
        let tangent = direction(anchor, vertices[next(edge_start)])?;
        loop {
            let following = next(contact);
            // Project the step itself to avoid subtracting two large heights.
            let gain = projection(tangent, vertices[contact], vertices[following], winding)?;
            if gain < 0.0 {
                break;
            }
            contact = following;
            advances += 1;
            // One circuit suffices for a strictly convex polygon. This also
            // bounds the loop if rounding obscures the end of a plateau.
            if advances > count {
                return Err(GeometryError::NumericalRange);
            }
        }

        let width = projection(tangent, anchor, vertices[contact], winding)?;
        if width <= 0.0 {
            return Err(GeometryError::NumericalRange);
        }
        let candidate = WidthWitness {
            width,
            edge_start,
            opposite_vertex: contact,
        };
        visit(candidate);
        if width < best.width {
            best = candidate;
        }
    }
    Ok((best, advances))
}

fn direction(from: Point, to: Point) -> Result<Point, GeometryError> {
    let delta = difference(to, from);
    let length = delta.x.hypot(delta.y);
    if length == 0.0 {
        return Err(GeometryError::ZeroLengthEdge);
    }
    if !length.is_finite() {
        return Err(GeometryError::NumericalRange);
    }
    Ok(Point {
        x: delta.x / length,
        y: delta.y / length,
    })
}

fn projection(
    tangent: Point,
    anchor: Point,
    point: Point,
    winding: f64,
) -> Result<f64, GeometryError> {
    let height = winding * cross(tangent, difference(point, anchor));
    if height.is_finite() {
        Ok(height)
    } else {
        Err(GeometryError::NumericalRange)
    }
}

fn difference(a: Point, b: Point) -> Point {
    Point {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - a.y * b.x
}
