use std::{env, error::Error, fmt::Write, fs, path::Path, process::ExitCode};

use caliper_slot::{Point, WidthWitness, minimum_width};

const USAGE: &str = "Usage: caliper-slot <polygon.txt> <width.svg>";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--help" || flag == "-h" => {
            println!("{USAGE}\n\nInput is one x y pair per line in convex boundary order.");
            ExitCode::SUCCESS
        }
        [input, output] => match run(Path::new(input), Path::new(output)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("caliper-slot: {error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run(input: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let vertices = parse_vertices(&source)?;
    let witness = minimum_width(&vertices)?;
    let svg = render_svg(&vertices, witness)?;
    fs::write(output, svg)
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    println!("Minimum width: {}", witness.width);
    Ok(())
}

fn parse_vertices(source: &str) -> Result<Vec<Point>, String> {
    let mut vertices = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let mut fields = line.split_whitespace();
        match (fields.next(), fields.next(), fields.next()) {
            (None, None, None) => continue,
            (Some(x), Some(y), None) => {
                let parse = |field: &str| {
                    field
                        .parse::<f64>()
                        .ok()
                        .filter(|value| value.is_finite())
                        .ok_or_else(|| format!("line {}: expected finite coordinates", index + 1))
                };
                vertices.push(Point {
                    x: parse(x)?,
                    y: parse(y)?,
                });
            }
            _ => return Err(format!("line {}: expected two coordinates", index + 1)),
        }
    }
    Ok(vertices)
}

fn render_svg(vertices: &[Point], witness: WidthWitness) -> Result<String, Box<dyn Error>> {
    let mut min = vertices[0];
    let mut max = vertices[0];
    for point in vertices {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    let dx = max.x - min.x;
    let dy = max.y - min.y;
    let span = dx.max(dy);
    if !span.is_finite() {
        return Err("polygon bounds exceed the SVG renderer's numeric range".into());
    }

    // Map the longer side to 400 pixels and center the polygon at (380, 280).
    let to_screen = |p: Point| Point {
        x: 380.0 + (p.x - min.x) / span * 400.0 - dx / span * 200.0,
        y: 280.0 - (p.y - min.y) / span * 400.0 + dy / span * 200.0,
    };
    let mut points = String::new();
    for &point in vertices {
        let p = to_screen(point);
        write!(points, "{:.6},{:.6} ", p.x, p.y)?;
    }

    let a = vertices[witness.edge_start];
    let b = vertices[(witness.edge_start + 1) % vertices.len()];
    let edge_x = b.x - a.x;
    let edge_y = b.y - a.y;
    let length = edge_x.hypot(edge_y);
    let tangent = Point {
        x: edge_x / length,
        y: -edge_y / length,
    };
    let anchor = to_screen(a);
    let endpoint = to_screen(b);
    let contact = to_screen(vertices[witness.opposite_vertex]);
    let normal = Point {
        x: -tangent.y,
        y: tangent.x,
    };
    let gap = (contact.x - anchor.x) * normal.x + (contact.y - anchor.y) * normal.y;
    let foot = Point {
        x: contact.x - gap * normal.x,
        y: contact.y - gap * normal.y,
    };
    let line = |a: Point, b: Point, class: &str| {
        format!(
            r#"<line class="{class}" x1="{:.6}" y1="{:.6}" x2="{:.6}" y2="{:.6}"/>"#,
            a.x, a.y, b.x, b.y
        )
    };
    // The endpoints extend beyond the viewport in either direction.
    let support = |p: Point| {
        line(
            Point {
                x: p.x - 960.0 * tangent.x,
                y: p.y - 960.0 * tangent.y,
            },
            Point {
                x: p.x + 960.0 * tangent.x,
                y: p.y + 960.0 * tangent.y,
            },
            "support",
        )
    };
    let reference = support(anchor);
    let opposite = support(contact);
    let edge = line(anchor, endpoint, "edge");
    let dimension = line(contact, foot, "dimension");
    let Point { x: cx, y: cy } = contact;
    let width = witness.width;
    let width_label = if (0.0001..1_000_000.0).contains(&width) {
        format!("{width:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    } else {
        format!("{width:.6e}")
    };

    Ok(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 560" role="img" aria-labelledby="title description">
  <title id="title">Minimum polygon width</title>
  <desc id="description">Two parallel support lines through the highlighted edge and marked vertex. The orange perpendicular measures the gap.</desc>
  <style>
    .outline {{ fill: #e9f1ff; stroke: #5276a5; stroke-width: 2; }}
    .support {{ stroke: #395477; stroke-width: 1.5; stroke-dasharray: 8 6; }}
    .edge {{ stroke: #173b68; stroke-width: 5; stroke-linecap: round; }}
    .dimension {{ stroke: #ac430d; stroke-width: 2.5; }}
    .label {{ font: 600 18px system-ui, sans-serif; fill: #172b49; stroke: #fff; stroke-width: 6; paint-order: stroke; }}
  </style>
  <rect width="760" height="560" fill="#fff"/>
  <polygon class="outline" points="{points}"/>
  {reference}
  {opposite}
  {dimension}
  {edge}
  <circle cx="{cx:.6}" cy="{cy:.6}" r="5.5" fill="#ac430d" stroke="#fff" stroke-width="1.5"/>
  <text class="label" x="24" y="34">Minimum width · {width_label}</text>
</svg>
"##
    ))
}
