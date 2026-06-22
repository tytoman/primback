use glam::{Vec2, Vec3};
pub mod geometry;

/// Internal geometry enum representing the raw shapes.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Geometry {
    Box {
        size: Vec3,
    },
    Sphere {
        radius: f32,
        rings: usize,
        sectors: usize,
    },
    IcoSphere {
        radius: f32,
        subdivisions: usize,
    },
    Cylinder {
        radius: f32,
        height: f32,
        sectors: usize,
    },
    Capsule {
        radius: f32,
        length: f32,
        rings: usize,
        sectors: usize,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
        radial_segments: usize,
        tubular_segments: usize,
    },
    Cone {
        radius: f32,
        height: f32,
        sectors: usize,
    },
    Pyramid {
        width: f32,
        depth: f32,
        height: f32,
    },
    Plane {
        size: Vec2,
        subdivisions: usize,
    },
    Disk {
        radius: f32,
        sectors: usize,
    },
}

#[inline]
fn canonical_bits(v: f32) -> u32 {
    if v.is_nan() {
        f32::NAN.to_bits()
    } else {
        v.to_bits()
    }
}

/// Returns `default` if `v` is NaN or infinite, otherwise returns `v`.
#[inline]
fn sanitize_f32(v: f32, default: f32) -> f32 {
    if v.is_finite() { v } else { default }
}

impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Geometry::Box { size: a }, Geometry::Box { size: b }) => {
                canonical_bits(a.x) == canonical_bits(b.x)
                    && canonical_bits(a.y) == canonical_bits(b.y)
                    && canonical_bits(a.z) == canonical_bits(b.z)
            }
            (
                Geometry::Sphere {
                    radius: r1,
                    rings: ri1,
                    sectors: s1,
                },
                Geometry::Sphere {
                    radius: r2,
                    rings: ri2,
                    sectors: s2,
                },
            ) => canonical_bits(*r1) == canonical_bits(*r2) && ri1 == ri2 && s1 == s2,
            (
                Geometry::IcoSphere {
                    radius: r1,
                    subdivisions: sd1,
                },
                Geometry::IcoSphere {
                    radius: r2,
                    subdivisions: sd2,
                },
            ) => canonical_bits(*r1) == canonical_bits(*r2) && sd1 == sd2,
            (
                Geometry::Cylinder {
                    radius: r1,
                    height: h1,
                    sectors: s1,
                },
                Geometry::Cylinder {
                    radius: r2,
                    height: h2,
                    sectors: s2,
                },
            ) => {
                canonical_bits(*r1) == canonical_bits(*r2)
                    && canonical_bits(*h1) == canonical_bits(*h2)
                    && s1 == s2
            }
            (
                Geometry::Capsule {
                    radius: r1,
                    length: l1,
                    rings: ri1,
                    sectors: s1,
                },
                Geometry::Capsule {
                    radius: r2,
                    length: l2,
                    rings: ri2,
                    sectors: s2,
                },
            ) => {
                canonical_bits(*r1) == canonical_bits(*r2)
                    && canonical_bits(*l1) == canonical_bits(*l2)
                    && ri1 == ri2
                    && s1 == s2
            }
            (
                Geometry::Torus {
                    major_radius: mr1,
                    minor_radius: mnr1,
                    radial_segments: rs1,
                    tubular_segments: ts1,
                },
                Geometry::Torus {
                    major_radius: mr2,
                    minor_radius: mnr2,
                    radial_segments: rs2,
                    tubular_segments: ts2,
                },
            ) => {
                canonical_bits(*mr1) == canonical_bits(*mr2)
                    && canonical_bits(*mnr1) == canonical_bits(*mnr2)
                    && rs1 == rs2
                    && ts1 == ts2
            }
            (
                Geometry::Cone {
                    radius: r1,
                    height: h1,
                    sectors: s1,
                },
                Geometry::Cone {
                    radius: r2,
                    height: h2,
                    sectors: s2,
                },
            ) => {
                canonical_bits(*r1) == canonical_bits(*r2)
                    && canonical_bits(*h1) == canonical_bits(*h2)
                    && s1 == s2
            }
            (
                Geometry::Pyramid {
                    width: w1,
                    depth: d1,
                    height: h1,
                },
                Geometry::Pyramid {
                    width: w2,
                    depth: d2,
                    height: h2,
                },
            ) => {
                canonical_bits(*w1) == canonical_bits(*w2)
                    && canonical_bits(*d1) == canonical_bits(*d2)
                    && canonical_bits(*h1) == canonical_bits(*h2)
            }
            (
                Geometry::Plane {
                    size: sz1,
                    subdivisions: sd1,
                },
                Geometry::Plane {
                    size: sz2,
                    subdivisions: sd2,
                },
            ) => {
                canonical_bits(sz1.x) == canonical_bits(sz2.x)
                    && canonical_bits(sz1.y) == canonical_bits(sz2.y)
                    && sd1 == sd2
            }
            (
                Geometry::Disk {
                    radius: r1,
                    sectors: s1,
                },
                Geometry::Disk {
                    radius: r2,
                    sectors: s2,
                },
            ) => canonical_bits(*r1) == canonical_bits(*r2) && s1 == s2,
            _ => false,
        }
    }
}

impl Eq for Geometry {}

impl std::hash::Hash for Geometry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Geometry::Box { size } => {
                canonical_bits(size.x).hash(state);
                canonical_bits(size.y).hash(state);
                canonical_bits(size.z).hash(state);
            }
            Geometry::Sphere {
                radius,
                rings,
                sectors,
            } => {
                canonical_bits(*radius).hash(state);
                rings.hash(state);
                sectors.hash(state);
            }
            Geometry::IcoSphere {
                radius,
                subdivisions,
            } => {
                canonical_bits(*radius).hash(state);
                subdivisions.hash(state);
            }
            Geometry::Cylinder {
                radius,
                height,
                sectors,
            } => {
                canonical_bits(*radius).hash(state);
                canonical_bits(*height).hash(state);
                sectors.hash(state);
            }
            Geometry::Capsule {
                radius,
                length,
                rings,
                sectors,
            } => {
                canonical_bits(*radius).hash(state);
                canonical_bits(*length).hash(state);
                rings.hash(state);
                sectors.hash(state);
            }
            Geometry::Torus {
                major_radius,
                minor_radius,
                radial_segments,
                tubular_segments,
            } => {
                canonical_bits(*major_radius).hash(state);
                canonical_bits(*minor_radius).hash(state);
                radial_segments.hash(state);
                tubular_segments.hash(state);
            }
            Geometry::Cone {
                radius,
                height,
                sectors,
            } => {
                canonical_bits(*radius).hash(state);
                canonical_bits(*height).hash(state);
                sectors.hash(state);
            }
            Geometry::Pyramid {
                width,
                depth,
                height,
            } => {
                canonical_bits(*width).hash(state);
                canonical_bits(*depth).hash(state);
                canonical_bits(*height).hash(state);
            }
            Geometry::Plane { size, subdivisions } => {
                canonical_bits(size.x).hash(state);
                canonical_bits(size.y).hash(state);
                subdivisions.hash(state);
            }
            Geometry::Disk { radius, sectors } => {
                canonical_bits(*radius).hash(state);
                sectors.hash(state);
            }
        }
    }
}

/// 3D primitive shapes supported by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shape {
    pub(crate) geometry: Geometry,
    pub(crate) lit: bool,
    pub(crate) inverted: bool,
}

impl Shape {
    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Returns whether the shape is lit.
    pub fn is_lit(&self) -> bool {
        self.lit
    }

    /// Sets whether the shape normal is inverted (inner-facing).
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Returns whether the shape normal is inverted (inner-facing).
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }
}

/// Builder for a cube (box) shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubeBuilder {
    pub size: Vec3,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for CubeBuilder {
    fn default() -> Self {
        Self {
            size: Vec3::ONE,
            lit: true,
            inverted: false,
        }
    }
}

impl CubeBuilder {
    /// Creates a default `CubeBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the size of the cube.
    pub fn size(mut self, size: Vec3) -> Self {
        self.size = size;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        use glam::Vec3;
        let size = Vec3::new(
            sanitize_f32(self.size.x, 1.0),
            sanitize_f32(self.size.y, 1.0),
            sanitize_f32(self.size.z, 1.0),
        );
        Shape {
            geometry: Geometry::Box { size },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<CubeBuilder> for Shape {
    fn from(builder: CubeBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a sphere shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereBuilder {
    pub radius: f32,
    pub rings: usize,
    pub sectors: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for SphereBuilder {
    fn default() -> Self {
        Self {
            radius: 1.0,
            rings: 16,
            sectors: 32,
            lit: true,
            inverted: false,
        }
    }
}

impl SphereBuilder {
    /// Creates a default `SphereBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the number of rings (latitude segments) (min: 1).
    pub fn rings(mut self, rings: usize) -> Self {
        self.rings = rings;
        self
    }

    /// Sets the number of sectors (longitude segments) (min: 3).
    pub fn sectors(mut self, sectors: usize) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Sphere {
                radius: sanitize_f32(self.radius, 1.0),
                rings: self.rings.max(1),
                sectors: self.sectors.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<SphereBuilder> for Shape {
    fn from(builder: SphereBuilder) -> Self {
        builder.build()
    }
}

/// Builder for an icosphere (geodesic sphere) shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IcoSphereBuilder {
    pub radius: f32,
    pub subdivisions: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for IcoSphereBuilder {
    fn default() -> Self {
        Self {
            radius: 1.0,
            subdivisions: 2,
            lit: true,
            inverted: false,
        }
    }
}

impl IcoSphereBuilder {
    /// Creates a default `IcoSphereBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the number of subdivisions (max: 8).
    pub fn subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::IcoSphere {
                radius: sanitize_f32(self.radius, 1.0),
                subdivisions: self.subdivisions.min(8),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<IcoSphereBuilder> for Shape {
    fn from(builder: IcoSphereBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a cylinder shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CylinderBuilder {
    pub radius: f32,
    pub height: f32,
    pub sectors: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for CylinderBuilder {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            sectors: 16,
            lit: true,
            inverted: false,
        }
    }
}

impl CylinderBuilder {
    /// Creates a default `CylinderBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets the number of sectors (tessellation) (min: 3).
    pub fn sectors(mut self, sectors: usize) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Cylinder {
                radius: sanitize_f32(self.radius, 0.5),
                height: sanitize_f32(self.height, 1.0),
                sectors: self.sectors.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<CylinderBuilder> for Shape {
    fn from(builder: CylinderBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a capsule shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapsuleBuilder {
    pub radius: f32,
    pub length: f32,
    pub rings: usize,
    pub sectors: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for CapsuleBuilder {
    fn default() -> Self {
        Self {
            radius: 0.5,
            length: 1.0,
            rings: 8,
            sectors: 16,
            lit: true,
            inverted: false,
        }
    }
}

impl CapsuleBuilder {
    /// Creates a default `CapsuleBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the radius of the caps.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the height of the cylindrical body.
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Sets the number of rings in the caps (min: 1).
    pub fn rings(mut self, rings: usize) -> Self {
        self.rings = rings;
        self
    }

    /// Sets the number of sectors (min: 3).
    pub fn sectors(mut self, sectors: usize) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Capsule {
                radius: sanitize_f32(self.radius, 0.5),
                length: sanitize_f32(self.length, 1.0),
                rings: self.rings.max(1),
                sectors: self.sectors.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<CapsuleBuilder> for Shape {
    fn from(builder: CapsuleBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a torus shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TorusBuilder {
    pub major_radius: f32,
    pub minor_radius: f32,
    pub radial_segments: usize,
    pub tubular_segments: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for TorusBuilder {
    fn default() -> Self {
        Self {
            major_radius: 1.0,
            minor_radius: 0.25,
            radial_segments: 16,
            tubular_segments: 8,
            lit: true,
            inverted: false,
        }
    }
}

impl TorusBuilder {
    /// Creates a default `TorusBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the major radius (distance from the center of the tube to the center of the torus).
    pub fn major_radius(mut self, major_radius: f32) -> Self {
        self.major_radius = major_radius;
        self
    }

    /// Sets the minor radius (radius of the tube).
    pub fn minor_radius(mut self, minor_radius: f32) -> Self {
        self.minor_radius = minor_radius;
        self
    }

    /// Sets the radial segments count (min: 3).
    pub fn radial_segments(mut self, radial_segments: usize) -> Self {
        self.radial_segments = radial_segments;
        self
    }

    /// Sets the tubular segments count (min: 3).
    pub fn tubular_segments(mut self, tubular_segments: usize) -> Self {
        self.tubular_segments = tubular_segments;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Torus {
                major_radius: sanitize_f32(self.major_radius, 1.0),
                minor_radius: sanitize_f32(self.minor_radius, 0.25),
                radial_segments: self.radial_segments.max(3),
                tubular_segments: self.tubular_segments.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<TorusBuilder> for Shape {
    fn from(builder: TorusBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a cone shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConeBuilder {
    pub radius: f32,
    pub height: f32,
    pub sectors: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for ConeBuilder {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            sectors: 16,
            lit: true,
            inverted: false,
        }
    }
}

impl ConeBuilder {
    /// Creates a default `ConeBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets the base sectors count (min: 3).
    pub fn sectors(mut self, sectors: usize) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Cone {
                radius: sanitize_f32(self.radius, 0.5),
                height: sanitize_f32(self.height, 1.0),
                sectors: self.sectors.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<ConeBuilder> for Shape {
    fn from(builder: ConeBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a pyramid shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PyramidBuilder {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for PyramidBuilder {
    fn default() -> Self {
        Self {
            width: 1.0,
            depth: 1.0,
            height: 1.0,
            lit: true,
            inverted: false,
        }
    }
}

impl PyramidBuilder {
    /// Creates a default `PyramidBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the base depth.
    pub fn depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Pyramid {
                width: sanitize_f32(self.width, 1.0),
                depth: sanitize_f32(self.depth, 1.0),
                height: sanitize_f32(self.height, 1.0),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<PyramidBuilder> for Shape {
    fn from(builder: PyramidBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a plane shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaneBuilder {
    pub size: Vec2,
    pub subdivisions: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for PlaneBuilder {
    fn default() -> Self {
        Self {
            size: Vec2::ONE,
            subdivisions: 1,
            lit: true,
            inverted: false,
        }
    }
}

impl PlaneBuilder {
    /// Creates a default `PlaneBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the plane size.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Sets the subdivision count (min: 1).
    pub fn subdivisions(mut self, subdivisions: usize) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        use glam::Vec2;
        let size = Vec2::new(
            sanitize_f32(self.size.x, 1.0),
            sanitize_f32(self.size.y, 1.0),
        );
        Shape {
            geometry: Geometry::Plane {
                size,
                subdivisions: self.subdivisions.max(1),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<PlaneBuilder> for Shape {
    fn from(builder: PlaneBuilder) -> Self {
        builder.build()
    }
}

/// Builder for a disk shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiskBuilder {
    pub radius: f32,
    pub sectors: usize,
    pub lit: bool,
    pub inverted: bool,
}

impl Default for DiskBuilder {
    fn default() -> Self {
        Self {
            radius: 1.0,
            sectors: 16,
            lit: true,
            inverted: false,
        }
    }
}

impl DiskBuilder {
    /// Creates a default `DiskBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the radius.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the base sectors count (min: 3).
    pub fn sectors(mut self, sectors: usize) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets whether the shape is lit.
    pub fn lit(mut self, lit: bool) -> Self {
        self.lit = lit;
        self
    }

    /// Disables lighting for this shape.
    pub fn unlit(mut self) -> Self {
        self.lit = false;
        self
    }

    /// Sets whether the shape normal is inverted.
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Builds the `Shape`.
    pub fn build(self) -> Shape {
        Shape {
            geometry: Geometry::Disk {
                radius: sanitize_f32(self.radius, 1.0),
                sectors: self.sectors.max(3),
            },
            lit: self.lit,
            inverted: self.inverted,
        }
    }
}

impl From<DiskBuilder> for Shape {
    fn from(builder: DiskBuilder) -> Self {
        builder.build()
    }
}

impl Shape {
    /// Returns a default `CubeBuilder`.
    pub fn cube() -> CubeBuilder {
        CubeBuilder::default()
    }

    /// Returns a default `SphereBuilder`.
    pub fn sphere() -> SphereBuilder {
        SphereBuilder::default()
    }

    /// Returns a default `IcoSphereBuilder`.
    pub fn ico_sphere() -> IcoSphereBuilder {
        IcoSphereBuilder::default()
    }

    /// Returns a default `CylinderBuilder`.
    pub fn cylinder() -> CylinderBuilder {
        CylinderBuilder::default()
    }

    /// Returns a default `CapsuleBuilder`.
    pub fn capsule() -> CapsuleBuilder {
        CapsuleBuilder::default()
    }

    /// Returns a default `TorusBuilder`.
    pub fn torus() -> TorusBuilder {
        TorusBuilder::default()
    }

    /// Returns a default `ConeBuilder`.
    pub fn cone() -> ConeBuilder {
        ConeBuilder::default()
    }

    /// Returns a default `PyramidBuilder`.
    pub fn pyramid() -> PyramidBuilder {
        PyramidBuilder::default()
    }

    /// Returns a default `PlaneBuilder`.
    pub fn plane() -> PlaneBuilder {
        PlaneBuilder::default()
    }

    /// Returns a default `DiskBuilder`.
    pub fn disk() -> DiskBuilder {
        DiskBuilder::default()
    }
}
