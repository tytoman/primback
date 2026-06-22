use crate::primback::{
    graphics::vertices::InstanceVertex,
    types::shape::{Geometry, Shape},
};
use glam::{Vec3, vec2, vec3};

/// Generates solid geometry (vertices and indices) for the shape in the local coordinate system.
pub fn generate_solid_geometry(shape: &Shape) -> (Vec<InstanceVertex>, Vec<u32>) {
    let mut vertices = Vec::new();

    match &shape.geometry {
        Geometry::Plane { size, subdivisions } => {
            let subs = (*subdivisions).max(1);
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;
            let normal = [0.0, 1.0, 0.0];

            let mut pts = Vec::with_capacity((subs + 1) * (subs + 1));
            for z_i in 0..=subs {
                let z_frac = z_i as f32 / subs as f32;
                let z = -half_h + size.y * z_frac;
                for x_i in 0..=subs {
                    let x_frac = x_i as f32 / subs as f32;
                    let x = -half_w + size.x * x_frac;
                    pts.push(vec3(x, 0.0, z));
                }
            }

            for z_i in 0..subs {
                for x_i in 0..subs {
                    let p0 = z_i * (subs + 1) + x_i;
                    let p1 = z_i * (subs + 1) + x_i + 1;
                    let p2 = (z_i + 1) * (subs + 1) + x_i + 1;
                    let p3 = (z_i + 1) * (subs + 1) + x_i;

                    let quad = [p0, p3, p2, p0, p2, p1];
                    for &idx in &quad {
                        vertices.push(InstanceVertex {
                            position: pts[idx].into(),
                            normal,
                        });
                    }
                }
            }
        }
        Geometry::Disk { radius, sectors } => {
            let sectors = (*sectors).max(3);
            let normal = [0.0, 1.0, 0.0];
            let center = InstanceVertex {
                position: [0.0, 0.0, 0.0],
                normal,
            };

            let mut pts = Vec::with_capacity(sectors);
            for i in 0..sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let local_pt = vec3(radius * theta.cos(), 0.0, radius * theta.sin());
                pts.push(InstanceVertex {
                    position: local_pt.into(),
                    normal,
                });
            }

            for i in 0..sectors {
                let next_i = (i + 1) % sectors;
                vertices.push(center);
                vertices.push(pts[next_i]);
                vertices.push(pts[i]);
            }
        }
        Geometry::Box { size } => {
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;
            let half_d = size.z * 0.5;

            let v = [
                vec3(-half_w, -half_h, -half_d),
                vec3(half_w, -half_h, -half_d),
                vec3(half_w, half_h, -half_d),
                vec3(-half_w, half_h, -half_d),
                vec3(-half_w, -half_h, half_d),
                vec3(half_w, -half_h, half_d),
                vec3(half_w, half_h, half_d),
                vec3(-half_w, half_h, half_d),
            ];

            let add_quad = |verts: &mut Vec<InstanceVertex>,
                            p1: Vec3,
                            p2: Vec3,
                            p3: Vec3,
                            p4: Vec3,
                            n: Vec3| {
                let normal = n.into();
                let quad = [p1, p2, p3, p1, p3, p4];
                for &p in &quad {
                    verts.push(InstanceVertex {
                        position: p.into(),
                        normal,
                    });
                }
            };

            // Front face (Z+), normal: [0, 0, 1]
            add_quad(&mut vertices, v[4], v[5], v[6], v[7], Vec3::Z);
            // Back face (Z-), normal: [0, 0, -1]
            add_quad(&mut vertices, v[1], v[0], v[3], v[2], -Vec3::Z);
            // Left face (X-), normal: [-1, 0, 0]
            add_quad(&mut vertices, v[0], v[4], v[7], v[3], -Vec3::X);
            // Right face (X+), normal: [1, 0, 0]
            add_quad(&mut vertices, v[5], v[1], v[2], v[6], Vec3::X);
            // Top face (Y+), normal: [0, 1, 0]
            add_quad(&mut vertices, v[7], v[6], v[2], v[3], Vec3::Y);
            // Bottom face (Y-), normal: [0, -1, 0]
            add_quad(&mut vertices, v[0], v[1], v[5], v[4], -Vec3::Y);
        }
        Geometry::Sphere {
            radius,
            rings,
            sectors,
        } => {
            let rings = (*rings).max(1);
            let sectors = (*sectors).max(3);
            let mut grid = Vec::new();
            let mut normals = Vec::new();

            for r in 0..=rings {
                let phi = std::f32::consts::PI * (r as f32) / (rings as f32);
                let y = phi.cos();
                let r_sin = phi.sin();
                let mut ring_pts = Vec::new();
                let mut ring_norms = Vec::new();
                for s in 0..=sectors {
                    let theta = 2.0 * std::f32::consts::PI * (s as f32) / (sectors as f32);
                    let x = theta.cos() * r_sin;
                    let z = theta.sin() * r_sin;

                    let local_normal = vec3(x, y, z);
                    let pt = local_normal * *radius;
                    ring_pts.push(pt);
                    ring_norms.push(local_normal);
                }
                grid.push(ring_pts);
                normals.push(ring_norms);
            }

            for r in 0..rings {
                for s in 0..sectors {
                    let p1 = grid[r][s];
                    let p2 = grid[r][s + 1];
                    let p3 = grid[r + 1][s + 1];
                    let p4 = grid[r + 1][s];

                    let n1 = normals[r][s];
                    let n2 = normals[r][s + 1];
                    let n3 = normals[r + 1][s + 1];
                    let n4 = normals[r + 1][s];

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p2.into(),
                        normal: n2.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p4.into(),
                        normal: n4.into(),
                    });
                }
            }
        }
        Geometry::IcoSphere {
            radius,
            subdivisions,
        } => {
            let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
            let mut base_vertices = vec![
                vec3(-1.0, phi, 0.0).normalize(),
                vec3(1.0, phi, 0.0).normalize(),
                vec3(-1.0, -phi, 0.0).normalize(),
                vec3(1.0, -phi, 0.0).normalize(),
                vec3(0.0, -1.0, phi).normalize(),
                vec3(0.0, 1.0, phi).normalize(),
                vec3(0.0, -1.0, -phi).normalize(),
                vec3(0.0, 1.0, -phi).normalize(),
                vec3(phi, 0.0, -1.0).normalize(),
                vec3(phi, 0.0, 1.0).normalize(),
                vec3(-phi, 0.0, -1.0).normalize(),
                vec3(-phi, 0.0, 1.0).normalize(),
            ];

            let mut triangles = vec![
                (0, 11, 5),
                (0, 5, 1),
                (0, 1, 7),
                (0, 7, 10),
                (0, 10, 11),
                (1, 5, 9),
                (5, 11, 4),
                (11, 10, 2),
                (10, 7, 6),
                (7, 1, 8),
                (3, 9, 4),
                (3, 4, 2),
                (3, 2, 6),
                (3, 6, 8),
                (3, 8, 9),
                (4, 9, 5),
                (2, 4, 11),
                (6, 2, 10),
                (8, 6, 7),
                (9, 8, 1),
            ];

            let mut midpoint_cache = std::collections::HashMap::new();
            let get_midpoint = |v1: usize,
                                v2: usize,
                                base_verts: &mut Vec<Vec3>,
                                cache: &mut std::collections::HashMap<(usize, usize), usize>|
             -> usize {
                let key = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                if let Some(&idx) = cache.get(&key) {
                    return idx;
                }
                let p1 = base_verts[v1];
                let p2 = base_verts[v2];
                let midpoint = (p1 + p2).normalize();
                let new_idx = base_verts.len();
                base_verts.push(midpoint);
                cache.insert(key, new_idx);
                new_idx
            };

            for _ in 0..*subdivisions {
                let mut new_triangles = Vec::new();
                for (v1, v2, v3) in triangles {
                    let a = get_midpoint(v1, v2, &mut base_vertices, &mut midpoint_cache);
                    let b = get_midpoint(v2, v3, &mut base_vertices, &mut midpoint_cache);
                    let c = get_midpoint(v3, v1, &mut base_vertices, &mut midpoint_cache);

                    new_triangles.push((v1, a, c));
                    new_triangles.push((v2, b, a));
                    new_triangles.push((v3, c, b));
                    new_triangles.push((a, b, c));
                }
                triangles = new_triangles;
            }

            for (v1, v2, v3) in triangles {
                let p1 = base_vertices[v1];
                let p2 = base_vertices[v2];
                let p3 = base_vertices[v3];

                vertices.push(InstanceVertex {
                    position: (p1 * *radius).into(),
                    normal: p1.into(),
                });
                vertices.push(InstanceVertex {
                    position: (p2 * *radius).into(),
                    normal: p2.into(),
                });
                vertices.push(InstanceVertex {
                    position: (p3 * *radius).into(),
                    normal: p3.into(),
                });
            }
        }
        Geometry::Cylinder {
            radius,
            height,
            sectors,
        } => {
            let sectors = (*sectors).max(3);
            let half_h = *height * 0.5;

            let mut side_pts = Vec::new();
            let mut side_norms = Vec::new();
            for i in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let cos = theta.cos();
                let sin = theta.sin();
                side_pts.push(vec2(cos * *radius, sin * *radius));
                side_norms.push(vec3(cos, 0.0, sin).normalize());
            }

            for i in 0..sectors {
                let p1_bottom = vec3(side_pts[i].x, -half_h, side_pts[i].y);
                let p1_top = vec3(side_pts[i].x, half_h, side_pts[i].y);
                let p2_bottom = vec3(side_pts[i + 1].x, -half_h, side_pts[i + 1].y);
                let p2_top = vec3(side_pts[i + 1].x, half_h, side_pts[i + 1].y);

                let n1 = side_norms[i].into();
                let n2 = side_norms[i + 1].into();

                // Triangle 1: p1_bottom -> p1_top -> p2_top
                vertices.push(InstanceVertex {
                    position: p1_bottom.into(),
                    normal: n1,
                });
                vertices.push(InstanceVertex {
                    position: p1_top.into(),
                    normal: n1,
                });
                vertices.push(InstanceVertex {
                    position: p2_top.into(),
                    normal: n2,
                });

                // Triangle 2: p1_bottom -> p2_top -> p2_bottom
                vertices.push(InstanceVertex {
                    position: p1_bottom.into(),
                    normal: n1,
                });
                vertices.push(InstanceVertex {
                    position: p2_top.into(),
                    normal: n2,
                });
                vertices.push(InstanceVertex {
                    position: p2_bottom.into(),
                    normal: n2,
                });
            }

            let top_center = vec3(0.0, half_h, 0.0);
            let top_normal = [0.0, 1.0, 0.0];
            for i in 0..sectors {
                let p1 = vec3(side_pts[i].x, half_h, side_pts[i].y);
                let p2 = vec3(side_pts[i + 1].x, half_h, side_pts[i + 1].y);

                vertices.push(InstanceVertex {
                    position: top_center.into(),
                    normal: top_normal,
                });
                vertices.push(InstanceVertex {
                    position: p2.into(),
                    normal: top_normal,
                });
                vertices.push(InstanceVertex {
                    position: p1.into(),
                    normal: top_normal,
                });
            }

            let bottom_center = vec3(0.0, -half_h, 0.0);
            let bottom_normal = [0.0, -1.0, 0.0];
            for i in 0..sectors {
                let p1 = vec3(side_pts[i].x, -half_h, side_pts[i].y);
                let p2 = vec3(side_pts[i + 1].x, -half_h, side_pts[i + 1].y);

                vertices.push(InstanceVertex {
                    position: bottom_center.into(),
                    normal: bottom_normal,
                });
                vertices.push(InstanceVertex {
                    position: p1.into(),
                    normal: bottom_normal,
                });
                vertices.push(InstanceVertex {
                    position: p2.into(),
                    normal: bottom_normal,
                });
            }
        }
        Geometry::Capsule {
            radius,
            length,
            rings,
            sectors,
        } => {
            let radius = *radius;
            let rings = *rings;
            let sectors = *sectors;
            let cyl_h = (length - 2.0 * radius).max(0.0);
            let mut grid = Vec::new();
            let mut normals = Vec::new();
            let half_rings = rings / 2;

            for r in 0..=(rings + 1) {
                let (phi, y_offset) = if r <= half_rings {
                    let phi = std::f32::consts::PI * (r as f32) / (rings as f32);
                    (phi, cyl_h * 0.5)
                } else {
                    let phi = std::f32::consts::PI * ((r - 1) as f32) / (rings as f32);
                    (phi, -cyl_h * 0.5)
                };

                let y_local = phi.cos();
                let r_sin = phi.sin();
                let y = y_local * radius + y_offset;

                let mut ring_pts = Vec::new();
                let mut ring_norms = Vec::new();
                for s in 0..=sectors {
                    let theta = 2.0 * std::f32::consts::PI * (s as f32) / (sectors as f32);
                    let x = theta.cos() * r_sin;
                    let z = theta.sin() * r_sin;

                    let local_normal = vec3(x, y_local, z).normalize();
                    let local_pt = vec3(x * radius, y, z * radius);

                    ring_pts.push(local_pt);
                    ring_norms.push(local_normal);
                }
                grid.push(ring_pts);
                normals.push(ring_norms);
            }

            for r in 0..(rings + 1) {
                for s in 0..sectors {
                    let p1 = grid[r][s];
                    let p2 = grid[r][s + 1];
                    let p3 = grid[r + 1][s + 1];
                    let p4 = grid[r + 1][s];

                    let n1 = normals[r][s];
                    let n2 = normals[r][s + 1];
                    let n3 = normals[r + 1][s + 1];
                    let n4 = normals[r + 1][s];

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p2.into(),
                        normal: n2.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p4.into(),
                        normal: n4.into(),
                    });
                }
            }
        }
        Geometry::Torus {
            major_radius,
            minor_radius,
            radial_segments,
            tubular_segments,
        } => {
            let radial_segments = (*radial_segments).max(3);
            let tubular_segments = (*tubular_segments).max(3);
            let mut grid = Vec::new();
            let mut normals = Vec::new();

            for u in 0..=radial_segments {
                let theta = 2.0 * std::f32::consts::PI * (u as f32) / (radial_segments as f32);
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();

                let mut ring_pts = Vec::new();
                let mut ring_norms = Vec::new();

                for v in 0..=tubular_segments {
                    let phi = 2.0 * std::f32::consts::PI * (v as f32) / (tubular_segments as f32);
                    let cos_phi = phi.cos();
                    let sin_phi = phi.sin();

                    let x = (major_radius + minor_radius * cos_phi) * cos_theta;
                    let y = minor_radius * sin_phi;
                    let z = (major_radius + minor_radius * cos_phi) * sin_theta;

                    let local_pt = vec3(x, y, z);
                    let local_normal =
                        vec3(cos_phi * cos_theta, sin_phi, cos_phi * sin_theta).normalize();

                    ring_pts.push(local_pt);
                    ring_norms.push(local_normal);
                }
                grid.push(ring_pts);
                normals.push(ring_norms);
            }

            for u in 0..radial_segments {
                for v in 0..tubular_segments {
                    let p1 = grid[u][v];
                    let p2 = grid[u][v + 1];
                    let p3 = grid[u + 1][v + 1];
                    let p4 = grid[u + 1][v];

                    let n1 = normals[u][v];
                    let n2 = normals[u][v + 1];
                    let n3 = normals[u + 1][v + 1];
                    let n4 = normals[u + 1][v];

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p2.into(),
                        normal: n2.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });

                    vertices.push(InstanceVertex {
                        position: p1.into(),
                        normal: n1.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p3.into(),
                        normal: n3.into(),
                    });
                    vertices.push(InstanceVertex {
                        position: p4.into(),
                        normal: n4.into(),
                    });
                }
            }
        }
        Geometry::Cone {
            radius,
            height,
            sectors,
        } => {
            let radius = *radius;
            let sectors = (*sectors).max(3);
            let half_h = *height * 0.5;

            let length_hypot = (radius * radius + *height * *height).sqrt();
            let sin_beta = if length_hypot > 0.0 {
                radius / length_hypot
            } else {
                0.0
            };
            let cos_beta = if length_hypot > 0.0 {
                *height / length_hypot
            } else {
                1.0
            };

            let mut base_pts = Vec::new();
            let mut base_norms = Vec::new();

            for i in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let cos = theta.cos();
                let sin = theta.sin();

                let local_normal = vec3(cos * cos_beta, sin_beta, sin * cos_beta).normalize();
                base_pts.push(vec3(radius * cos, -half_h, radius * sin));
                base_norms.push(local_normal);
            }

            let apex = vec3(0.0, half_h, 0.0);

            // Side faces
            for i in 0..sectors {
                let p1 = base_pts[i];
                let p2 = base_pts[i + 1];
                let n1 = base_norms[i];
                let n2 = base_norms[i + 1];
                let n_apex = ((n1 + n2) * 0.5).normalize();

                vertices.push(InstanceVertex {
                    position: p1.into(),
                    normal: n1.into(),
                });
                vertices.push(InstanceVertex {
                    position: apex.into(),
                    normal: n_apex.into(),
                });
                vertices.push(InstanceVertex {
                    position: p2.into(),
                    normal: n2.into(),
                });
            }

            // Bottom cap
            let bottom_center = vec3(0.0, -half_h, 0.0);
            let bottom_normal = [0.0, -1.0, 0.0];
            for i in 0..sectors {
                vertices.push(InstanceVertex {
                    position: bottom_center.into(),
                    normal: bottom_normal,
                });
                vertices.push(InstanceVertex {
                    position: base_pts[i].into(),
                    normal: bottom_normal,
                });
                vertices.push(InstanceVertex {
                    position: base_pts[i + 1].into(),
                    normal: bottom_normal,
                });
            }
        }
        Geometry::Pyramid {
            width,
            depth,
            height,
        } => {
            let half_w = *width * 0.5;
            let half_d = *depth * 0.5;
            let half_h = *height * 0.5;

            let v = [
                vec3(-half_w, -half_h, -half_d),
                vec3(half_w, -half_h, -half_d),
                vec3(half_w, -half_h, half_d),
                vec3(-half_w, -half_h, half_d),
            ];
            let apex = vec3(0.0, half_h, 0.0);

            // Side face 0 (Back, Z-)
            let n0 = (v[0] - v[1]).cross(apex - v[1]).normalize_or_zero().into();
            vertices.push(InstanceVertex {
                position: v[1].into(),
                normal: n0,
            });
            vertices.push(InstanceVertex {
                position: v[0].into(),
                normal: n0,
            });
            vertices.push(InstanceVertex {
                position: apex.into(),
                normal: n0,
            });

            // Side face 1 (Right, X+)
            let n1 = (v[1] - v[2]).cross(apex - v[2]).normalize_or_zero().into();
            vertices.push(InstanceVertex {
                position: v[2].into(),
                normal: n1,
            });
            vertices.push(InstanceVertex {
                position: v[1].into(),
                normal: n1,
            });
            vertices.push(InstanceVertex {
                position: apex.into(),
                normal: n1,
            });

            // Side face 2 (Front, Z+)
            let n2 = (v[2] - v[3]).cross(apex - v[3]).normalize_or_zero().into();
            vertices.push(InstanceVertex {
                position: v[3].into(),
                normal: n2,
            });
            vertices.push(InstanceVertex {
                position: v[2].into(),
                normal: n2,
            });
            vertices.push(InstanceVertex {
                position: apex.into(),
                normal: n2,
            });

            // Side face 3 (Left, X-)
            let n3 = (v[3] - v[0]).cross(apex - v[0]).normalize_or_zero().into();
            vertices.push(InstanceVertex {
                position: v[0].into(),
                normal: n3,
            });
            vertices.push(InstanceVertex {
                position: v[3].into(),
                normal: n3,
            });
            vertices.push(InstanceVertex {
                position: apex.into(),
                normal: n3,
            });

            // Bottom cap
            let bottom_normal = [0.0, -1.0, 0.0];
            vertices.push(InstanceVertex {
                position: v[0].into(),
                normal: bottom_normal,
            });
            vertices.push(InstanceVertex {
                position: v[1].into(),
                normal: bottom_normal,
            });
            vertices.push(InstanceVertex {
                position: v[2].into(),
                normal: bottom_normal,
            });

            vertices.push(InstanceVertex {
                position: v[0].into(),
                normal: bottom_normal,
            });
            vertices.push(InstanceVertex {
                position: v[2].into(),
                normal: bottom_normal,
            });
            vertices.push(InstanceVertex {
                position: v[3].into(),
                normal: bottom_normal,
            });
        }
    }

    if shape.inverted {
        for v in &mut vertices {
            v.normal = [-v.normal[0], -v.normal[1], -v.normal[2]];
        }
    }

    if !shape.lit {
        for v in &mut vertices {
            v.normal = [0.0, 0.0, 0.0];
        }
    }

    let (unique_vertices, mut indices) = deduplicate_vertices(vertices);

    if shape.inverted {
        for chunk in indices.chunks_exact_mut(3) {
            chunk.swap(1, 2);
        }
    }

    (unique_vertices, indices)
}

/// Generates wireframe geometry (vertices and indices) for the shape in the local coordinate system.
pub fn generate_wire_geometry(shape: &Shape) -> (Vec<InstanceVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let normal = [0.0, 0.0, 0.0];

    match &shape.geometry {
        Geometry::Plane { size, subdivisions } => {
            let subs = (*subdivisions).max(1);
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;

            let mut pts = Vec::with_capacity((subs + 1) * (subs + 1));
            for z_i in 0..=subs {
                let z_frac = z_i as f32 / subs as f32;
                let z = -half_h + size.y * z_frac;
                for x_i in 0..=subs {
                    let x_frac = x_i as f32 / subs as f32;
                    let x = -half_w + size.x * x_frac;
                    pts.push(vec3(x, 0.0, z));
                }
            }

            for z_i in 0..=subs {
                for x_i in 0..subs {
                    let p1 = z_i * (subs + 1) + x_i;
                    let p2 = z_i * (subs + 1) + x_i + 1;
                    vertices.push(InstanceVertex {
                        position: pts[p1].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: pts[p2].into(),
                        normal,
                    });
                }
            }
            for x_i in 0..=subs {
                for z_i in 0..subs {
                    let p1 = z_i * (subs + 1) + x_i;
                    let p2 = (z_i + 1) * (subs + 1) + x_i;
                    vertices.push(InstanceVertex {
                        position: pts[p1].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: pts[p2].into(),
                        normal,
                    });
                }
            }
        }
        Geometry::Disk { radius, sectors } => {
            let sectors = (*sectors).max(3);
            let center = InstanceVertex {
                position: [0.0, 0.0, 0.0],
                normal,
            };

            let mut pts = Vec::with_capacity(sectors);
            for i in 0..sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let local_pt = vec3(radius * theta.cos(), 0.0, radius * theta.sin());
                pts.push(InstanceVertex {
                    position: local_pt.into(),
                    normal,
                });
            }

            for i in 0..sectors {
                let next_i = (i + 1) % sectors;
                vertices.push(pts[i]);
                vertices.push(pts[next_i]);
                vertices.push(center);
                vertices.push(pts[i]);
            }
        }
        Geometry::Box { size } => {
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;
            let half_d = size.z * 0.5;

            let v = [
                vec3(-half_w, -half_h, -half_d),
                vec3(half_w, -half_h, -half_d),
                vec3(half_w, half_h, -half_d),
                vec3(-half_w, half_h, -half_d),
                vec3(-half_w, -half_h, half_d),
                vec3(half_w, -half_h, half_d),
                vec3(half_w, half_h, half_d),
                vec3(-half_w, half_h, half_d),
            ];

            let edges = [
                (0, 1),
                (1, 5),
                (5, 4),
                (4, 0),
                (3, 2),
                (2, 6),
                (6, 7),
                (7, 3),
                (0, 3),
                (1, 2),
                (5, 6),
                (4, 7),
            ];

            for &(p1, p2) in &edges {
                vertices.push(InstanceVertex {
                    position: v[p1].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: v[p2].into(),
                    normal,
                });
            }
        }
        Geometry::Sphere {
            radius,
            rings,
            sectors,
        } => {
            let rings = (*rings).max(1);
            let sectors = (*sectors).max(3);
            let mut grid = Vec::new();

            for r in 0..=rings {
                let phi = std::f32::consts::PI * (r as f32) / (rings as f32);
                let y = phi.cos();
                let r_sin = phi.sin();
                let mut ring = Vec::new();
                for s in 0..=sectors {
                    let theta = 2.0 * std::f32::consts::PI * (s as f32) / (sectors as f32);
                    let x = theta.cos() * r_sin;
                    let z = theta.sin() * r_sin;
                    let local_pt = vec3(x, y, z) * *radius;
                    ring.push(local_pt);
                }
                grid.push(ring);
            }

            for r in 0..=rings {
                for s in 0..sectors {
                    vertices.push(InstanceVertex {
                        position: grid[r][s].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[r][s + 1].into(),
                        normal,
                    });
                }
            }

            for r in 0..rings {
                for s in 0..=sectors {
                    vertices.push(InstanceVertex {
                        position: grid[r][s].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[r + 1][s].into(),
                        normal,
                    });
                }
            }
        }
        Geometry::IcoSphere {
            radius,
            subdivisions,
        } => {
            let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
            let mut base_vertices = vec![
                vec3(-1.0, phi, 0.0).normalize(),
                vec3(1.0, phi, 0.0).normalize(),
                vec3(-1.0, -phi, 0.0).normalize(),
                vec3(1.0, -phi, 0.0).normalize(),
                vec3(0.0, -1.0, phi).normalize(),
                vec3(0.0, 1.0, phi).normalize(),
                vec3(0.0, -1.0, -phi).normalize(),
                vec3(0.0, 1.0, -phi).normalize(),
                vec3(phi, 0.0, -1.0).normalize(),
                vec3(phi, 0.0, 1.0).normalize(),
                vec3(-phi, 0.0, -1.0).normalize(),
                vec3(-phi, 0.0, 1.0).normalize(),
            ];

            let mut triangles = vec![
                (0, 11, 5),
                (0, 5, 1),
                (0, 1, 7),
                (0, 7, 10),
                (0, 10, 11),
                (1, 5, 9),
                (5, 11, 4),
                (11, 10, 2),
                (10, 7, 6),
                (7, 1, 8),
                (3, 9, 4),
                (3, 4, 2),
                (3, 2, 6),
                (3, 6, 8),
                (3, 8, 9),
                (4, 9, 5),
                (2, 4, 11),
                (6, 2, 10),
                (8, 6, 7),
                (9, 8, 1),
            ];

            let mut midpoint_cache = std::collections::HashMap::new();
            let get_midpoint = |v1: usize,
                                v2: usize,
                                base_verts: &mut Vec<Vec3>,
                                cache: &mut std::collections::HashMap<(usize, usize), usize>|
             -> usize {
                let key = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                if let Some(&idx) = cache.get(&key) {
                    return idx;
                }
                let p1 = base_verts[v1];
                let p2 = base_verts[v2];
                let midpoint = (p1 + p2).normalize();
                let new_idx = base_verts.len();
                base_verts.push(midpoint);
                cache.insert(key, new_idx);
                new_idx
            };

            for _ in 0..*subdivisions {
                let mut new_triangles = Vec::new();
                for (v1, v2, v3) in triangles {
                    let a = get_midpoint(v1, v2, &mut base_vertices, &mut midpoint_cache);
                    let b = get_midpoint(v2, v3, &mut base_vertices, &mut midpoint_cache);
                    let c = get_midpoint(v3, v1, &mut base_vertices, &mut midpoint_cache);

                    new_triangles.push((v1, a, c));
                    new_triangles.push((v2, b, a));
                    new_triangles.push((v3, c, b));
                    new_triangles.push((a, b, c));
                }
                triangles = new_triangles;
            }

            let mut edges = std::collections::HashSet::new();
            for (v1, v2, v3) in triangles {
                let mut add_edge = |a, b| {
                    let key = if a < b { (a, b) } else { (b, a) };
                    edges.insert(key);
                };
                add_edge(v1, v2);
                add_edge(v2, v3);
                add_edge(v3, v1);
            }

            for (v1, v2) in edges {
                let p1 = base_vertices[v1] * *radius;
                let p2 = base_vertices[v2] * *radius;
                vertices.push(InstanceVertex {
                    position: p1.into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: p2.into(),
                    normal,
                });
            }
        }
        Geometry::Cylinder {
            radius,
            height,
            sectors,
        } => {
            let sectors = (*sectors).max(3);
            let half_h = *height * 0.5;

            let mut base_ring = Vec::new();
            let mut top_ring = Vec::new();

            for i in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let cos = theta.cos();
                let sin = theta.sin();

                base_ring.push(vec3(cos * *radius, -half_h, sin * *radius));
                top_ring.push(vec3(cos * *radius, half_h, sin * *radius));
            }

            for i in 0..sectors {
                vertices.push(InstanceVertex {
                    position: base_ring[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: base_ring[i + 1].into(),
                    normal,
                });

                vertices.push(InstanceVertex {
                    position: top_ring[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: top_ring[i + 1].into(),
                    normal,
                });

                vertices.push(InstanceVertex {
                    position: base_ring[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: top_ring[i].into(),
                    normal,
                });
            }
        }
        Geometry::Capsule {
            radius,
            length,
            rings,
            sectors,
        } => {
            let radius = *radius;
            let rings = *rings;
            let sectors = *sectors;
            let cyl_h = (length - 2.0 * radius).max(0.0);
            let mut grid = Vec::new();
            let half_rings = rings / 2;

            for r in 0..=(rings + 1) {
                let (phi, y_offset) = if r <= half_rings {
                    let phi = std::f32::consts::PI * (r as f32) / (rings as f32);
                    (phi, cyl_h * 0.5)
                } else {
                    let phi = std::f32::consts::PI * ((r - 1) as f32) / (rings as f32);
                    (phi, -cyl_h * 0.5)
                };

                let y_local = phi.cos();
                let r_sin = phi.sin();
                let y = y_local * radius + y_offset;

                let mut ring = Vec::new();
                for s in 0..=sectors {
                    let theta = 2.0 * std::f32::consts::PI * (s as f32) / (sectors as f32);
                    let x = theta.cos() * r_sin;
                    let z = theta.sin() * r_sin;
                    let local_pt = vec3(x * radius, y, z * radius);
                    ring.push(local_pt);
                }
                grid.push(ring);
            }

            for r in 0..=(rings + 1) {
                for s in 0..sectors {
                    vertices.push(InstanceVertex {
                        position: grid[r][s].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[r][s + 1].into(),
                        normal,
                    });
                }
            }

            for r in 0..(rings + 1) {
                for s in 0..=sectors {
                    vertices.push(InstanceVertex {
                        position: grid[r][s].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[r + 1][s].into(),
                        normal,
                    });
                }
            }
        }
        Geometry::Torus {
            major_radius,
            minor_radius,
            radial_segments,
            tubular_segments,
        } => {
            let radial_segments = (*radial_segments).max(3);
            let tubular_segments = (*tubular_segments).max(3);
            let mut grid = Vec::new();

            for u in 0..=radial_segments {
                let theta = 2.0 * std::f32::consts::PI * (u as f32) / (radial_segments as f32);
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();

                let mut ring = Vec::new();
                for v in 0..=tubular_segments {
                    let phi = 2.0 * std::f32::consts::PI * (v as f32) / (tubular_segments as f32);
                    let cos_phi = phi.cos();
                    let sin_phi = phi.sin();

                    let x = (major_radius + minor_radius * cos_phi) * cos_theta;
                    let y = minor_radius * sin_phi;
                    let z = (major_radius + minor_radius * cos_phi) * sin_theta;

                    ring.push(vec3(x, y, z));
                }
                grid.push(ring);
            }

            for u in 0..=radial_segments {
                for v in 0..tubular_segments {
                    vertices.push(InstanceVertex {
                        position: grid[u][v].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[u][v + 1].into(),
                        normal,
                    });
                }
            }

            for u in 0..radial_segments {
                for v in 0..=tubular_segments {
                    vertices.push(InstanceVertex {
                        position: grid[u][v].into(),
                        normal,
                    });
                    vertices.push(InstanceVertex {
                        position: grid[u + 1][v].into(),
                        normal,
                    });
                }
            }
        }
        Geometry::Cone {
            radius,
            height,
            sectors,
        } => {
            let radius = *radius;
            let sectors = (*sectors).max(3);
            let half_h = *height * 0.5;

            let mut base_ring = Vec::new();
            for i in 0..=sectors {
                let theta = 2.0 * std::f32::consts::PI * (i as f32) / (sectors as f32);
                let cos = theta.cos();
                let sin = theta.sin();
                base_ring.push(vec3(radius * cos, -half_h, radius * sin));
            }

            let apex = vec3(0.0, half_h, 0.0);

            for i in 0..sectors {
                vertices.push(InstanceVertex {
                    position: base_ring[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: base_ring[i + 1].into(),
                    normal,
                });

                vertices.push(InstanceVertex {
                    position: base_ring[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: apex.into(),
                    normal,
                });
            }
        }
        Geometry::Pyramid {
            width,
            depth,
            height,
        } => {
            let half_w = *width * 0.5;
            let half_d = *depth * 0.5;
            let half_h = *height * 0.5;

            let v = [
                vec3(-half_w, -half_h, -half_d),
                vec3(half_w, -half_h, -half_d),
                vec3(half_w, -half_h, half_d),
                vec3(-half_w, -half_h, half_d),
            ];
            let apex = vec3(0.0, half_h, 0.0);

            for i in 0..4 {
                vertices.push(InstanceVertex {
                    position: v[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: v[(i + 1) % 4].into(),
                    normal,
                });

                vertices.push(InstanceVertex {
                    position: v[i].into(),
                    normal,
                });
                vertices.push(InstanceVertex {
                    position: apex.into(),
                    normal,
                });
            }
        }
    }

    deduplicate_vertices(vertices)
}

/// Helper function to deduplicate vertices and generate an index buffer.
fn deduplicate_vertices(vertices: Vec<InstanceVertex>) -> (Vec<InstanceVertex>, Vec<u32>) {
    use std::collections::HashMap;

    let mut unique_vertices = Vec::new();
    let mut index_map = HashMap::with_capacity(vertices.len());
    let mut indices = Vec::with_capacity(vertices.len());

    #[derive(Copy, Clone)]
    struct Key(InstanceVertex);

    impl PartialEq for Key {
        fn eq(&self, other: &Self) -> bool {
            let pos_eq = self
                .0
                .position
                .iter()
                .zip(other.0.position.iter())
                .all(|(a, b)| a.to_bits() == b.to_bits());
            let norm_eq = self
                .0
                .normal
                .iter()
                .zip(other.0.normal.iter())
                .all(|(a, b)| a.to_bits() == b.to_bits());
            pos_eq && norm_eq
        }
    }

    impl Eq for Key {}

    impl std::hash::Hash for Key {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            for &val in &self.0.position {
                val.to_bits().hash(state);
            }
            for &val in &self.0.normal {
                val.to_bits().hash(state);
            }
        }
    }

    for vertex in vertices {
        let key = Key(vertex);
        let next_idx = unique_vertices.len() as u32;
        let &mut idx = index_map.entry(key).or_insert(next_idx);
        indices.push(idx);
        if idx == next_idx {
            unique_vertices.push(vertex);
        }
    }

    (unique_vertices, indices)
}
