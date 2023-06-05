use bevy::{prelude::Mesh as BMesh, render::render_resource::PrimitiveTopology};
use tmf::{HandenesType, TMFMesh};

pub trait Tmf2Bevy {
    fn into_bevy(self) -> BMesh;
}
pub trait Bevy2Tmf {
    fn into_tmf(self) -> tmf::TMFMesh;
}
impl Tmf2Bevy for tmf::TMFMesh {
    fn into_bevy(mut self) -> BMesh {
        use bevy::render::mesh::Indices;
        use bevy::render::mesh::VertexAttributeValues::{Float32x2, Float32x3, Float32x4};

        let mut mesh = BMesh::new(PrimitiveTopology::TriangleList);

        if let Some(positions) = self.set_vertices(vec![]) {
            mesh.insert_attribute(BMesh::ATTRIBUTE_POSITION, Float32x3(Conv(positions).into()));
        }
        if let Some(normals) = self.set_normals(vec![]) {
            mesh.insert_attribute(BMesh::ATTRIBUTE_NORMAL, Float32x3(Conv(normals).into()));
        }
        if let Some(uvs) = self.set_uvs(vec![]) {
            mesh.insert_attribute(BMesh::ATTRIBUTE_UV_0, Float32x2(Conv(uvs).into()));
        }
        if let Some(tangents) = self.set_tangents(vec![]) {
            mesh.insert_attribute(BMesh::ATTRIBUTE_TANGENT, Float32x4(Conv(tangents).into()));
        }
        if let Some(indices) = self.set_vertex_triangles(vec![]) {
            mesh.set_indices(Some(Indices::U32(indices)));
        }
        mesh
    }
}
impl Bevy2Tmf for BMesh {
    fn into_tmf(mut self) -> tmf::TMFMesh {
        use bevy::render::mesh::Indices;
        use bevy::render::mesh::VertexAttributeValues::{Float32x2, Float32x3, Float32x4};

        let mut mesh = TMFMesh::default();

        if let Some(Float32x3(positions)) = self.remove_attribute(BMesh::ATTRIBUTE_POSITION) {
            mesh.set_vertices(Conv(positions).into());
        }
        if let Some(Float32x3(normals)) = self.remove_attribute(BMesh::ATTRIBUTE_NORMAL) {
            mesh.set_normals(Conv(normals).into());
        }
        if let Some(Float32x2(uvs)) = self.remove_attribute(BMesh::ATTRIBUTE_UV_0) {
            mesh.set_uvs(Conv(uvs).into());
        }
        if let Some(Float32x4(tangents)) = self.remove_attribute(BMesh::ATTRIBUTE_TANGENT) {
            mesh.set_tangents(Conv(tangents).into());
        }
        if let Some(Indices::U32(indices)) = self.indices() {
            mesh.set_vertex_triangles(indices.clone());
            mesh.set_normal_triangles(indices.clone());
            mesh.set_uv_triangles(indices.clone());
            mesh.set_tangent_triangles(indices.clone());
        }
        mesh.optimize();

        mesh
    }
}
struct Conv<T>(T);

// From (…) to [… ; N] aka TMF -> bevy
impl From<Conv<Vec<(f32, f32)>>> for Vec<[f32; 2]> {
    fn from(value: Conv<Vec<(f32, f32)>>) -> Self {
        value.0.into_iter().map(|(x, y)| [x, y]).collect()
    }
}
impl From<Conv<Vec<(f32, f32, f32)>>> for Vec<[f32; 3]> {
    fn from(value: Conv<Vec<(f32, f32, f32)>>) -> Self {
        value.0.into_iter().map(|(x, y, z)| [x, y, z]).collect()
    }
}
impl From<Conv<Vec<((f32, f32, f32), HandenesType)>>> for Vec<[f32; 4]> {
    fn from(value: Conv<Vec<((f32, f32, f32), HandenesType)>>) -> Self {
        value
            .0
            .into_iter()
            .map(|((x, y, z), w)| [x, y, z, w.into()])
            .collect()
    }
}

// From [… ; N] to (…) aka bevy -> TMF
impl Conv<Vec<[f32; 2]>> {
    fn into(self) -> Vec<(f32, f32)> {
        self.0.into_iter().map(|[x, y]| (x, y)).collect()
    }
}
impl Conv<Vec<[f32; 3]>> {
    fn into(self) -> Vec<(f32, f32, f32)> {
        self.0.into_iter().map(|[x, y, z]| (x, y, z)).collect()
    }
}
impl Conv<Vec<[f32; 4]>> {
    fn into(self) -> Vec<((f32, f32, f32), HandenesType)> {
        self.0
            .into_iter()
            .map(|[x, y, z, w]| ((x, y, z), w.into()))
            .collect()
    }
}
