//! Meshing behaviour through the public API: hidden faces, greedy merging, AO and seams.

#![expect(
    clippy::expect_used,
    reason = "test helpers may panic with a clear message"
)]

use proptest::prelude::*;
use voxel_core::{Chunk, LocalPos, VoxelId};
use voxel_mesh::{Face, MeshBuffers, PaddedChunk, Quad, mesh_quads};

const STONE: VoxelId = VoxelId(3);
const GRASS: VoxelId = VoxelId(1);
const OPEN: [u8; 4] = [3; 4];

fn local(x: u8, y: u8, z: u8) -> LocalPos {
    LocalPos::new(x, y, z).expect("in bounds")
}

fn chunk_with(voxels: &[((u8, u8, u8), VoxelId)]) -> Chunk {
    let mut chunk = Chunk::empty();
    for &((x, y, z), voxel) in voxels {
        chunk.set(local(x, y, z), voxel);
    }
    chunk
}

fn alone(chunk: &Chunk) -> PaddedChunk {
    PaddedChunk::from_chunks(chunk, |_| None)
}

fn quads_facing(quads: &[Quad], face: Face) -> Vec<Quad> {
    quads
        .iter()
        .copied()
        .filter(|quad| quad.face == face)
        .collect()
}

fn area(quads: &[Quad]) -> u32 {
    quads
        .iter()
        .map(|quad| u32::from(quad.size[0]) * u32::from(quad.size[1]))
        .sum()
}

#[test]
fn an_empty_chunk_has_no_faces() {
    assert_eq!(mesh_quads(&alone(&Chunk::empty())), Vec::<Quad>::new());
}

#[test]
fn a_lone_voxel_shows_six_open_faces() {
    let quads = mesh_quads(&alone(&chunk_with(&[((4, 5, 6), STONE)])));

    assert_eq!(quads.len(), 6);
    for face in Face::ALL {
        let facing = quads_facing(&quads, face);
        assert_eq!(facing.len(), 1, "{face:?}");
        assert_eq!(facing[0].position, [4, 5, 6]);
        assert_eq!(facing[0].size, [1, 1]);
        assert_eq!(facing[0].voxel, STONE);
        assert_eq!(facing[0].ao, OPEN);
    }
}

#[test]
fn touching_voxels_hide_their_shared_faces_and_merge_the_rest() {
    let quads = mesh_quads(&alone(&chunk_with(&[
        ((4, 5, 6), STONE),
        ((5, 5, 6), STONE),
    ])));

    assert_eq!(quads.len(), 6);
    assert_eq!(area(&quads), 10);
}

#[test]
fn different_materials_never_merge() {
    let quads = mesh_quads(&alone(&chunk_with(&[
        ((4, 5, 6), STONE),
        ((5, 5, 6), GRASS),
    ])));

    assert_eq!(quads_facing(&quads, Face::PosY).len(), 2);
    assert_eq!(area(&quads), 10);
}

#[test]
fn a_full_chunk_in_the_open_is_six_large_quads() {
    let full = Chunk::from_fn(|_| STONE);

    let quads = mesh_quads(&alone(&full));

    assert_eq!(quads.len(), 6);
    assert!(quads.iter().all(|quad| quad.size == [32, 32]));
}

#[test]
fn a_chunk_buried_in_solid_neighbours_has_no_faces() {
    let full = Chunk::from_fn(|_| STONE);

    let padded = PaddedChunk::from_chunks(&full, |_| Some(&full));

    assert_eq!(mesh_quads(&padded), Vec::<Quad>::new());
}

#[test]
fn a_solid_neighbour_hides_the_faces_at_the_seam_only() {
    let full = Chunk::from_fn(|_| STONE);

    let padded = PaddedChunk::from_chunks(&full, |offset| (offset == [1, 0, 0]).then_some(&full));
    let quads = mesh_quads(&padded);

    assert_eq!(quads_facing(&quads, Face::PosX), Vec::<Quad>::new());
    assert_eq!(quads.len(), 5);
}

#[test]
fn a_neighbouring_voxel_darkens_the_corners_next_to_it() {
    // A floor voxel with a wall voxel above its +x neighbour: the floor's top face has its
    // two corners at x = 1 next to the wall.
    let quads = mesh_quads(&alone(&chunk_with(&[
        ((0, 0, 0), STONE),
        ((1, 1, 0), STONE),
    ])));

    let top = quads_facing(&quads, Face::PosY);
    let floor_top = top
        .iter()
        .find(|quad| quad.position == [0, 0, 0])
        .expect("floor top face");

    // Corner order is (u0 v0, u1 v0, u1 v1, u0 v1); for +y faces u is z and v is x.
    assert_eq!(floor_top.ao, [3, 3, 2, 2]);
}

#[test]
fn faces_with_different_occlusion_do_not_merge() {
    // A row of floor along z with a wall voxel above one end.
    let quads = mesh_quads(&alone(&chunk_with(&[
        ((0, 0, 0), STONE),
        ((0, 0, 1), STONE),
        ((0, 0, 2), STONE),
        ((0, 0, 3), STONE),
        ((0, 1, 3), STONE),
    ])));

    let floor_top: Vec<Quad> = quads_facing(&quads, Face::PosY)
        .into_iter()
        .filter(|quad| quad.position[1] == 0)
        .collect();

    // z = 0 and 1 are open and merge; z = 2 touches the wall and stays apart; z = 3 is covered.
    assert_eq!(floor_top.len(), 2);
    assert!(
        floor_top
            .iter()
            .any(|quad| quad.ao == OPEN && quad.size[0] == 2)
    );
}

#[test]
fn buffers_have_four_vertices_and_two_triangles_per_quad() {
    let quads = mesh_quads(&alone(&chunk_with(&[((4, 5, 6), STONE)])));

    let buffers = MeshBuffers::from_quads(&quads);

    assert_eq!(buffers.positions.len(), 24);
    assert_eq!(buffers.normals.len(), 24);
    assert_eq!(buffers.ao.len(), 24);
    assert_eq!(buffers.voxels.len(), 24);
    assert_eq!(buffers.indices.len(), 36);
    let top_normals = buffers
        .normals
        .iter()
        .filter(|normal| normal[1] > 0.5)
        .count();
    assert_eq!(top_normals, 4);
}

#[test]
fn every_triangle_faces_out_of_the_voxel() {
    let quads = mesh_quads(&alone(&chunk_with(&[((4, 5, 6), STONE)])));

    let buffers = MeshBuffers::from_quads(&quads);

    for triangle in buffers.indices.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|i| buffers.positions[triangle[i] as usize]);
        let normal = buffers.normals[triangle[0] as usize];
        let (ab, ac) = (sub(b, a), sub(c, a));
        let winding = cross(ab, ac);
        assert!(
            dot(winding, normal) > 0.0,
            "triangle {triangle:?} faces inward"
        );
    }
}

#[test]
fn quads_split_along_their_darker_diagonal() {
    let quads = [Quad {
        face: Face::PosY,
        position: [0, 0, 0],
        size: [1, 1],
        voxel: STONE,
        ao: [3, 0, 3, 3],
    }];

    let buffers = MeshBuffers::from_quads(&quads);

    // The darkest corner (index 1) must be on the shared edge of both triangles.
    for triangle in buffers.indices.as_chunks::<3>().0 {
        assert!(triangle.contains(&1), "{triangle:?}");
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A deterministic scatter of voxels: about one in `1 / density` positions is solid.
fn scattered_chunk(seed: u64, density: u64) -> Chunk {
    Chunk::from_fn(|pos| {
        let mut hash = seed ^ (pos.index() as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        hash = (hash ^ (hash >> 31)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        hash ^= hash >> 29;
        if hash.is_multiple_of(density) {
            VoxelId(if hash & (1 << 40) == 0 { 1 } else { 2 })
        } else {
            VoxelId::AIR
        }
    })
}

/// Visible faces counted voxel by voxel, the slow obvious way: outside the chunk is air.
fn visible_faces(chunk: &Chunk, face: Face) -> Vec<[u8; 3]> {
    let normal = face.normal();
    LocalPos::all()
        .filter(|&pos| !chunk.get(pos).is_air())
        .filter(|&pos| {
            let coords = [pos.x(), pos.y(), pos.z()];
            let next = [0, 1, 2]
                .map(|axis| u8::try_from(i16::from(coords[axis]) + i16::from(normal[axis])).ok());
            match next {
                [Some(x), Some(y), Some(z)] => {
                    LocalPos::new(x, y, z).map_or(true, |next| chunk.get(next).is_air())
                }
                _ => true,
            }
        })
        .map(|pos| [pos.x(), pos.y(), pos.z()])
        .collect()
}

/// Voxels whose `face` a quad covers, one entry per covered face.
fn covered_faces(quad: &Quad) -> Vec<[u8; 3]> {
    let (u_axis, v_axis) = quad.face.plane_axes();
    let mut covered = Vec::new();
    for du in 0..quad.size[0] {
        for dv in 0..quad.size[1] {
            let mut pos = quad.position;
            pos[u_axis] += du;
            pos[v_axis] += dv;
            covered.push(pos);
        }
    }
    covered
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn quads_cover_every_visible_face_exactly_once(seed: u64, density in 1u64..6) {
        let chunk = scattered_chunk(seed, density);

        let quads = mesh_quads(&alone(&chunk));

        for face in Face::ALL {
            let mut covered: Vec<[u8; 3]> = quads_facing(&quads, face)
                .iter()
                .flat_map(covered_faces)
                .collect();
            let mut expected = visible_faces(&chunk, face);
            covered.sort_unstable();
            expected.sort_unstable();
            prop_assert_eq!(covered, expected, "{:?}", face);
        }
    }

    #[test]
    fn merged_quads_keep_the_material_of_every_face(seed: u64, density in 1u64..6) {
        let chunk = scattered_chunk(seed, density);

        for quad in mesh_quads(&alone(&chunk)) {
            for [x, y, z] in covered_faces(&quad) {
                prop_assert_eq!(chunk.get(local(x, y, z)), quad.voxel);
            }
        }
    }
}
