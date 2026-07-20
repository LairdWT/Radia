use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub const UDF_MAGIC: [u8; 8] = *b"RADIAUDF";
pub const UDF_VERSION: u32 = 1;
const LEAF_TRIANGLES: usize = 8;
const MODEL_EXTENT_METERS: f64 = 2.8;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Point3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3 {
    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn component(self, axis: usize) -> f64 {
        match axis {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length_squared(self) -> f64 {
        self.dot(self)
    }

    fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl std::ops::Add for Point3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Point3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Point3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[derive(Clone, Copy, Debug)]
struct Triangle {
    vertices: [u32; 3],
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: Point3,
    max: Point3,
}

impl Bounds {
    fn empty() -> Self {
        Self {
            min: Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    fn include(&mut self, point: Point3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    fn include_bounds(&mut self, other: Self) {
        self.include(other.min);
        self.include(other.max);
    }

    fn extent(self) -> Point3 {
        self.max - self.min
    }

    fn distance_squared(self, point: Point3) -> f64 {
        let axis_distance = |value: f64, minimum: f64, maximum: f64| {
            if value < minimum {
                minimum - value
            } else if value > maximum {
                value - maximum
            } else {
                0.0
            }
        };
        let dx = axis_distance(point.x, self.min.x, self.max.x);
        let dy = axis_distance(point.y, self.min.y, self.max.y);
        let dz = axis_distance(point.z, self.min.z, self.max.z);
        dx * dx + dy * dy + dz * dz
    }
}

#[derive(Debug)]
struct Mesh {
    vertices: Vec<Point3>,
    triangles: Vec<Triangle>,
    source_bounds: Bounds,
}

#[derive(Clone, Copy, Debug)]
struct BvhNode {
    bounds: Bounds,
    left: u32,
    right: u32,
    start: u32,
    count: u32,
}

impl BvhNode {
    fn branch(bounds: Bounds, left: u32, right: u32) -> Self {
        Self {
            bounds,
            left,
            right,
            start: 0,
            count: 0,
        }
    }

    fn leaf(bounds: Bounds, start: usize, count: usize) -> Result<Self, String> {
        Ok(Self {
            bounds,
            left: 0,
            right: 0,
            start: u32::try_from(start).map_err(|_| "BVH leaf start exceeds u32".to_owned())?,
            count: u32::try_from(count).map_err(|_| "BVH leaf count exceeds u32".to_owned())?,
        })
    }

    const fn is_leaf(self) -> bool {
        self.count != 0
    }
}

#[derive(Debug)]
struct MeshBvh {
    vertices: Vec<Point3>,
    triangles: Vec<Triangle>,
    ordered_triangles: Vec<u32>,
    nodes: Vec<BvhNode>,
}

impl MeshBvh {
    fn build(mesh: Mesh) -> Result<Self, String> {
        let triangle_count = mesh.triangles.len();
        let mut bvh = Self {
            vertices: mesh.vertices,
            triangles: mesh.triangles,
            ordered_triangles: (0..triangle_count)
                .map(|index| u32::try_from(index).map_err(|_| "triangle count exceeds u32"))
                .collect::<Result<Vec<_>, _>>()?,
            nodes: Vec::with_capacity(triangle_count.saturating_mul(2)),
        };
        bvh.build_node(0, triangle_count)?;
        Ok(bvh)
    }

    fn build_node(&mut self, start: usize, end: usize) -> Result<u32, String> {
        let node_index =
            u32::try_from(self.nodes.len()).map_err(|_| "BVH node count exceeds u32".to_owned())?;
        self.nodes.push(BvhNode::leaf(Bounds::empty(), 0, 1)?);

        let mut bounds = Bounds::empty();
        let mut centroid_bounds = Bounds::empty();
        for ordered_index in &self.ordered_triangles[start..end] {
            let triangle = self.triangles[*ordered_index as usize];
            let triangle_bounds = self.triangle_bounds(triangle);
            bounds.include_bounds(triangle_bounds);
            centroid_bounds.include((triangle_bounds.min + triangle_bounds.max) * 0.5);
        }
        let count = end - start;
        if count <= LEAF_TRIANGLES {
            self.nodes[node_index as usize] = BvhNode::leaf(bounds, start, count)?;
            return Ok(node_index);
        }

        let extent = centroid_bounds.extent();
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        let midpoint = start + count / 2;
        let vertices = &self.vertices;
        let triangles = &self.triangles;
        self.ordered_triangles[start..end].select_nth_unstable_by(
            midpoint - start,
            |left, right| {
                triangle_centroid(vertices, triangles[*left as usize])
                    .component(axis)
                    .total_cmp(
                        &triangle_centroid(vertices, triangles[*right as usize]).component(axis),
                    )
            },
        );
        let left = self.build_node(start, midpoint)?;
        let right = self.build_node(midpoint, end)?;
        self.nodes[node_index as usize] = BvhNode::branch(bounds, left, right);
        Ok(node_index)
    }

    fn triangle_bounds(&self, triangle: Triangle) -> Bounds {
        let mut bounds = Bounds::empty();
        for index in triangle.vertices {
            bounds.include(self.vertices[index as usize]);
        }
        bounds
    }

    fn nearest_distance(&self, point: Point3) -> f64 {
        let mut best_squared = f64::INFINITY;
        let mut stack = vec![0_u32];
        while let Some(node_index) = stack.pop() {
            let node = self.nodes[node_index as usize];
            if node.bounds.distance_squared(point) >= best_squared {
                continue;
            }
            if node.is_leaf() {
                let start = node.start as usize;
                let end = start + node.count as usize;
                for ordered_index in &self.ordered_triangles[start..end] {
                    let triangle = self.triangles[*ordered_index as usize];
                    let [a, b, c] = triangle.vertices.map(|index| self.vertices[index as usize]);
                    best_squared =
                        best_squared.min(point_triangle_distance_squared(point, a, b, c));
                }
            } else {
                let left = self.nodes[node.left as usize];
                let right = self.nodes[node.right as usize];
                let left_distance = left.bounds.distance_squared(point);
                let right_distance = right.bounds.distance_squared(point);
                if left_distance < right_distance {
                    if right_distance < best_squared {
                        stack.push(node.right);
                    }
                    if left_distance < best_squared {
                        stack.push(node.left);
                    }
                } else {
                    if left_distance < best_squared {
                        stack.push(node.left);
                    }
                    if right_distance < best_squared {
                        stack.push(node.right);
                    }
                }
            }
        }
        best_squared.sqrt()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BakeReport {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub resolution: [u32; 3],
    pub source_min: [f64; 3],
    pub source_max: [f64; 3],
    pub volume_min: [f32; 3],
    pub volume_max: [f32; 3],
}

/// Parses an OBJ, normalizes it into meter-space, builds a BVH, and writes a
/// deterministic float32 unsigned-distance volume.
///
/// # Errors
///
/// Returns a descriptive error for invalid geometry, bounds, resolution, I/O,
/// numeric overflow, or worker failure.
pub fn bake_obj_file(input: &Path, output: &Path, resolution: u32) -> Result<BakeReport, String> {
    validate_resolution(resolution)?;
    let file = File::open(input).map_err(|error| format!("open {}: {error}", input.display()))?;
    let mut mesh = parse_obj(BufReader::new(file))?;
    let source_bounds = mesh.source_bounds;
    normalize_mesh(&mut mesh)?;
    let mesh_bounds = bounds_for_points(&mesh.vertices)?;
    let extent = mesh_bounds.extent();
    let maximum_extent = extent.x.max(extent.y).max(extent.z);
    let spacing_without_margin = maximum_extent / f64::from(resolution.saturating_sub(7));
    let margin = spacing_without_margin * 3.0;
    let volume_bounds = Bounds {
        min: mesh_bounds.min - Point3::new(margin, margin, margin),
        max: mesh_bounds.max + Point3::new(margin, margin, margin),
    };
    let vertex_count = mesh.vertices.len();
    let triangle_count = mesh.triangles.len();
    let bvh = Arc::new(MeshBvh::build(mesh)?);
    let samples = bake_volume(&bvh, volume_bounds, resolution)?;
    write_volume(output, volume_bounds, resolution, &samples)?;

    Ok(BakeReport {
        vertex_count,
        triangle_count,
        resolution: [resolution; 3],
        source_min: point_to_f64_array(source_bounds.min),
        source_max: point_to_f64_array(source_bounds.max),
        volume_min: point_to_f32_array(volume_bounds.min)?,
        volume_max: point_to_f32_array(volume_bounds.max)?,
    })
}

fn validate_resolution(resolution: u32) -> Result<(), String> {
    if !(32..=256).contains(&resolution) {
        return Err("resolution must be in 32..=256".to_owned());
    }
    Ok(())
}

fn parse_obj(reader: impl BufRead) -> Result<Mesh, String> {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut source_bounds = Bounds::empty();
    for (line_index, line) in reader.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.map_err(|error| format!("read OBJ line {line_number}: {error}"))?;
        let mut fields = line.split_ascii_whitespace();
        match fields.next() {
            Some("v") => {
                let point = Point3::new(
                    parse_f64(fields.next(), line_number, "vertex x")?,
                    parse_f64(fields.next(), line_number, "vertex y")?,
                    parse_f64(fields.next(), line_number, "vertex z")?,
                );
                if !point.is_finite() {
                    return Err(format!("OBJ line {line_number} has a non-finite vertex"));
                }
                source_bounds.include(point);
                vertices.push(point);
            }
            Some("f") => {
                let face = fields
                    .map(|field| parse_vertex_index(field, vertices.len(), line_number))
                    .collect::<Result<Vec<_>, _>>()?;
                if face.len() < 3 {
                    return Err(format!(
                        "OBJ line {line_number} has fewer than three vertices"
                    ));
                }
                for index in 1..face.len() - 1 {
                    triangles.push(Triangle {
                        vertices: [face[0], face[index], face[index + 1]],
                    });
                }
            }
            _ => {}
        }
    }
    if vertices.is_empty() || triangles.is_empty() {
        return Err("OBJ must contain finite vertices and faces".to_owned());
    }
    Ok(Mesh {
        vertices,
        triangles,
        source_bounds,
    })
}

fn parse_f64(value: Option<&str>, line: usize, field: &str) -> Result<f64, String> {
    value
        .ok_or_else(|| format!("OBJ line {line} is missing {field}"))?
        .parse::<f64>()
        .map_err(|_| format!("OBJ line {line} has invalid {field}"))
}

fn parse_vertex_index(field: &str, vertex_count: usize, line: usize) -> Result<u32, String> {
    let raw = field
        .split('/')
        .next()
        .ok_or_else(|| format!("OBJ line {line} has an empty face index"))?
        .parse::<i64>()
        .map_err(|_| format!("OBJ line {line} has an invalid face index"))?;
    if raw == 0 {
        return Err(format!("OBJ line {line} uses forbidden zero index"));
    }
    let count = i64::try_from(vertex_count).map_err(|_| "vertex count exceeds i64".to_owned())?;
    let resolved = if raw > 0 { raw - 1 } else { count + raw };
    if resolved < 0 || resolved >= count {
        return Err(format!("OBJ line {line} face index is out of range"));
    }
    u32::try_from(resolved).map_err(|_| "vertex index exceeds u32".to_owned())
}

fn normalize_mesh(mesh: &mut Mesh) -> Result<(), String> {
    let bounds = mesh.source_bounds;
    let extent = bounds.extent();
    let maximum_extent = extent.x.max(extent.y).max(extent.z);
    if !maximum_extent.is_finite() || maximum_extent <= 0.0 {
        return Err("OBJ bounds are degenerate".to_owned());
    }
    let scale = MODEL_EXTENT_METERS / maximum_extent;
    let center_x = (bounds.min.x + bounds.max.x) * 0.5;
    let center_z = (bounds.min.z + bounds.max.z) * 0.5;
    for vertex in &mut mesh.vertices {
        *vertex = Point3::new(
            (vertex.x - center_x) * scale,
            (vertex.y - bounds.min.y) * scale,
            (vertex.z - center_z) * scale,
        );
    }
    Ok(())
}

fn bounds_for_points(points: &[Point3]) -> Result<Bounds, String> {
    let mut bounds = Bounds::empty();
    for point in points {
        if !point.is_finite() {
            return Err("normalized OBJ contains a non-finite vertex".to_owned());
        }
        bounds.include(*point);
    }
    Ok(bounds)
}

fn bake_volume(bvh: &Arc<MeshBvh>, bounds: Bounds, resolution: u32) -> Result<Vec<f32>, String> {
    let resolution_usize = resolution as usize;
    let slice_length = resolution_usize
        .checked_mul(resolution_usize)
        .ok_or_else(|| "volume slice size overflow".to_owned())?;
    let sample_count = slice_length
        .checked_mul(resolution_usize)
        .ok_or_else(|| "volume sample count overflow".to_owned())?;
    let step = bounds.extent() * (1.0 / f64::from(resolution - 1));
    let worker_count = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(resolution_usize)
        .min(16);
    let mut handles = Vec::with_capacity(worker_count);
    for worker in 0..worker_count {
        let worker_bvh = Arc::clone(bvh);
        let worker_u32 = u32::try_from(worker).map_err(|_| "worker index exceeds u32")?;
        let worker_count_u32 =
            u32::try_from(worker_count).map_err(|_| "worker count exceeds u32")?;
        let z_start = worker_u32 * resolution / worker_count_u32;
        let z_end = (worker_u32 + 1) * resolution / worker_count_u32;
        handles.push(thread::spawn(move || -> Result<(u32, Vec<f32>), String> {
            let slice_count =
                usize::try_from(z_end - z_start).map_err(|_| "worker slice count exceeds usize")?;
            let mut values = Vec::with_capacity(slice_count * slice_length);
            for z in z_start..z_end {
                for y in 0..resolution {
                    for x in 0..resolution {
                        let point = Point3::new(
                            bounds.min.x + step.x * f64::from(x),
                            bounds.min.y + step.y * f64::from(y),
                            bounds.min.z + step.z * f64::from(z),
                        );
                        let distance = worker_bvh.nearest_distance(point);
                        let value = checked_f32(distance, "nearest-distance query")?;
                        values.push(value);
                    }
                }
            }
            Ok((z_start, values))
        }));
    }

    let mut chunks = Vec::with_capacity(worker_count);
    for handle in handles {
        chunks.push(
            handle
                .join()
                .map_err(|_| "distance worker panicked".to_owned())??,
        );
    }
    chunks.sort_unstable_by_key(|(z_start, _)| *z_start);
    let mut samples = Vec::with_capacity(sample_count);
    for (_, chunk) in chunks {
        samples.extend(chunk);
    }
    if samples.len() != sample_count {
        return Err("distance workers returned the wrong sample count".to_owned());
    }
    Ok(samples)
}

fn write_volume(
    output: &Path,
    bounds: Bounds,
    resolution: u32,
    samples: &[f32],
) -> Result<(), String> {
    let file =
        File::create(output).map_err(|error| format!("create {}: {error}", output.display()))?;
    let mut writer = BufWriter::new(file);
    writer
        .write_all(&UDF_MAGIC)
        .and_then(|()| writer.write_all(&UDF_VERSION.to_le_bytes()))
        .map_err(|error| format!("write {} header: {error}", output.display()))?;
    for value in [resolution; 3] {
        writer
            .write_all(&value.to_le_bytes())
            .map_err(|error| format!("write {} resolution: {error}", output.display()))?;
    }
    for value in [
        bounds.min.x,
        bounds.min.y,
        bounds.min.z,
        bounds.max.x,
        bounds.max.y,
        bounds.max.z,
    ] {
        let value = checked_f32(value, "volume bounds")?;
        writer
            .write_all(&value.to_le_bytes())
            .map_err(|error| format!("write {} bounds: {error}", output.display()))?;
    }
    let step = bounds.extent() * (1.0 / f64::from(resolution - 1));
    let conservative_error = 0.5 * (step.x * step.x + step.y * step.y + step.z * step.z).sqrt();
    let conservative_error = checked_f32(conservative_error, "volume error bound")?;
    writer
        .write_all(&conservative_error.to_le_bytes())
        .map_err(|error| format!("write {} error bound: {error}", output.display()))?;
    for sample in samples {
        writer
            .write_all(&sample.to_le_bytes())
            .map_err(|error| format!("write {} payload: {error}", output.display()))?;
    }
    writer
        .flush()
        .map_err(|error| format!("flush {}: {error}", output.display()))
}

fn triangle_centroid(vertices: &[Point3], triangle: Triangle) -> Point3 {
    let [a, b, c] = triangle.vertices.map(|index| vertices[index as usize]);
    (a + b + c) * (1.0 / 3.0)
}

#[allow(clippy::similar_names)]
fn point_triangle_distance_squared(
    point: Point3,
    vertex_a: Point3,
    vertex_b: Point3,
    vertex_c: Point3,
) -> f64 {
    let edge_ab = vertex_b - vertex_a;
    let edge_ac = vertex_c - vertex_a;
    let edge_bc = vertex_c - vertex_b;
    let maximum_edge_squared = edge_ab
        .length_squared()
        .max(edge_ac.length_squared())
        .max(edge_bc.length_squared());
    let area_squared = edge_ab.cross(edge_ac).length_squared();
    let degeneracy_bound = f64::EPSILON * maximum_edge_squared * maximum_edge_squared * 64.0;
    if area_squared <= degeneracy_bound {
        return point_segment_distance_squared(point, vertex_a, vertex_b)
            .min(point_segment_distance_squared(point, vertex_a, vertex_c))
            .min(point_segment_distance_squared(point, vertex_b, vertex_c));
    }

    let from_a = point - vertex_a;
    let d1 = edge_ab.dot(from_a);
    let d2 = edge_ac.dot(from_a);
    if d1 <= 0.0 && d2 <= 0.0 {
        return from_a.length_squared();
    }
    let from_b = point - vertex_b;
    let d3 = edge_ab.dot(from_b);
    let d4 = edge_ac.dot(from_b);
    if d3 >= 0.0 && d4 <= d3 {
        return from_b.length_squared();
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let barycentric_b = d1 / (d1 - d3);
        return (point - (vertex_a + edge_ab * barycentric_b)).length_squared();
    }
    let from_c = point - vertex_c;
    let d5 = edge_ab.dot(from_c);
    let d6 = edge_ac.dot(from_c);
    if d6 >= 0.0 && d5 <= d6 {
        return from_c.length_squared();
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let barycentric_c = d2 / (d2 - d6);
        return (point - (vertex_a + edge_ac * barycentric_c)).length_squared();
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let edge = vertex_c - vertex_b;
        let barycentric_c = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (point - (vertex_b + edge * barycentric_c)).length_squared();
    }
    let denominator = 1.0 / (va + vb + vc);
    let barycentric_b = vb * denominator;
    let barycentric_c = vc * denominator;
    (point - (vertex_a + edge_ab * barycentric_b + edge_ac * barycentric_c)).length_squared()
}

fn point_segment_distance_squared(point: Point3, a: Point3, b: Point3) -> f64 {
    let edge = b - a;
    let denominator = edge.length_squared();
    if denominator == 0.0 {
        return (point - a).length_squared();
    }
    let parameter = ((point - a).dot(edge) / denominator).clamp(0.0, 1.0);
    (point - (a + edge * parameter)).length_squared()
}

const fn point_to_f64_array(point: Point3) -> [f64; 3] {
    [point.x, point.y, point.z]
}

fn point_to_f32_array(point: Point3) -> Result<[f32; 3], String> {
    Ok([
        checked_f32(point.x, "point")?,
        checked_f32(point.y, "point")?,
        checked_f32(point.z, "point")?,
    ])
}

#[allow(clippy::cast_possible_truncation)]
fn checked_f32(value: f64, context: &str) -> Result<f32, String> {
    if !value.is_finite() || value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
        return Err(format!("{context} exceeds finite f32 range"));
    }
    Ok(value as f32)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{Point3, parse_obj, point_triangle_distance_squared};

    #[test]
    fn obj_parser_resolves_positive_and_negative_indices() {
        let source = b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1//1 2//2 3//3\nf -3 -2 -1\n";
        let mesh = parse_obj(Cursor::new(source)).expect("fixture is valid");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles.len(), 2);
        assert_eq!(mesh.triangles[0].vertices, [0, 1, 2]);
        assert_eq!(mesh.triangles[1].vertices, [0, 1, 2]);
    }

    #[test]
    fn triangle_distance_covers_face_edge_vertex_and_degenerate_cases() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(0.0, 1.0, 0.0);
        assert_close(
            point_triangle_distance_squared(Point3::new(0.25, 0.25, 2.0), a, b, c),
            4.0,
        );
        assert_close(
            point_triangle_distance_squared(Point3::new(-1.0, 0.0, 0.0), a, b, c),
            1.0,
        );
        assert_close(
            point_triangle_distance_squared(Point3::new(0.5, -1.0, 0.0), a, b, c),
            1.0,
        );
        assert_close(
            point_triangle_distance_squared(Point3::new(0.0, 0.0, 2.0), a, a, a),
            4.0,
        );
    }

    #[test]
    fn parser_rejects_non_finite_and_out_of_range_faces() {
        assert!(parse_obj(Cursor::new(b"v NaN 0 0\nf 1 1 1\n")).is_err());
        assert!(parse_obj(Cursor::new(b"v 0 0 0\nf 1 2 3\n")).is_err());
        assert!(parse_obj(Cursor::new(b"v 0 0 0\nf 0 0 0\n")).is_err());
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() <= f64::EPSILON * expected.abs().max(1.0) * 8.0);
    }
}
