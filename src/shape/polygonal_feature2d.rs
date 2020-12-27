use crate::math::{Isometry, Point, Real, Vector};
use crate::query::{self, ContactManifold, TrackedContact};
use crate::shape::{Cuboid, Segment};

#[derive(Debug)]
pub struct PolygonalFeature {
    pub vertices: [Point<Real>; 2],
    pub vids: [u8; 2],
    pub fid: u8,
    pub num_vertices: usize,
}

impl Default for PolygonalFeature {
    fn default() -> Self {
        Self {
            vertices: [Point::origin(); 2],
            vids: [0; 2],
            fid: 0,
            num_vertices: 0,
        }
    }
}

impl From<Segment> for PolygonalFeature {
    fn from(seg: Segment) -> Self {
        PolygonalFeature {
            vertices: [seg.a, seg.b],
            vids: [0, 2],
            fid: 1,
            num_vertices: 2,
        }
    }
}

impl PolygonalFeature {
    pub fn transform_by(&mut self, iso: &Isometry<Real>) {
        self.vertices[0] = iso * self.vertices[0];
        self.vertices[1] = iso * self.vertices[1];
    }

    pub fn contacts<ManifoldData, ContactData: Default + Copy>(
        pos12: &Isometry<f32>,
        pos21: &Isometry<f32>,
        sep_axis1: &Vector<f32>,
        sep_axis2: &Vector<f32>,
        feature1: &Self,
        feature2: &Self,
        prediction: f32,
        manifold: &mut ContactManifold<ManifoldData, ContactData>,
        flipped: bool,
    ) {
        match (feature1.num_vertices == 2, feature2.num_vertices == 2) {
            (true, true) => Self::face_face_contacts(
                pos12, feature1, sep_axis1, feature2, prediction, manifold, flipped,
            ),
            (true, false) => Self::face_vertex_contacts(
                pos12, feature1, sep_axis1, feature2, prediction, manifold, flipped,
            ),
            (false, true) => Self::face_vertex_contacts(
                pos21, feature2, sep_axis2, feature1, prediction, manifold, !flipped,
            ),
            (false, false) => unimplemented!(),
        }
    }

    /// Compute contacts points between a face and a vertex.
    ///
    /// This method assume we already know that at least one contact exists.
    pub fn face_vertex_contacts<ManifoldData, ContactData: Default + Copy>(
        pos12: &Isometry<Real>,
        face1: &Self,
        sep_axis1: &Vector<Real>,
        vertex2: &Self,
        _prediction: Real,
        manifold: &mut ContactManifold<ManifoldData, ContactData>,
        flipped: bool,
    ) {
        let v2_1 = pos12 * vertex2.vertices[0];
        let tangent1 = face1.vertices[1] - face1.vertices[0];
        let normal1 = Vector::new(-tangent1.y, tangent1.x);
        let denom = -normal1.dot(&sep_axis1);
        let dist = (face1.vertices[0] - v2_1).dot(&normal1) / denom;
        let local_p2 = v2_1;
        let local_p1 = v2_1 - dist * normal1;

        let contact = TrackedContact::flipped(
            local_p1,
            pos12.inverse_transform_point(&local_p2),
            face1.fid,
            vertex2.vids[0],
            dist,
            flipped,
        );

        manifold.points.push(contact);
    }

    pub fn face_face_contacts<ManifoldData, ContactData: Default + Copy>(
        pos12: &Isometry<Real>,
        face1: &Self,
        normal1: &Vector<Real>,
        face2: &Self,
        prediction: Real,
        manifold: &mut ContactManifold<ManifoldData, ContactData>,
        flipped: bool,
    ) {
        if let Some((clip_a, clip_b)) = query::details::clip_segment_segment_with_normal(
            (face1.vertices[0], face1.vertices[1]),
            (pos12 * face2.vertices[0], pos12 * face2.vertices[1]),
            *normal1,
        ) {
            let fids1 = [face1.vids[0], face1.fid, face1.vids[1]];
            let fids2 = [face2.vids[0], face2.fid, face2.vids[1]];

            let dist = (clip_a.1 - clip_a.0).dot(normal1);
            if true {
                // dist < prediction {
                let contact = TrackedContact::flipped(
                    clip_a.0,
                    pos12.inverse_transform_point(&clip_a.1),
                    fids1[clip_a.2],
                    fids2[clip_a.3],
                    dist,
                    flipped,
                );
                manifold.points.push(contact);
            }

            let dist = (clip_b.1 - clip_b.0).dot(normal1);
            if true {
                // dist < prediction {
                let contact = TrackedContact::flipped(
                    clip_b.0,
                    pos12.inverse_transform_point(&clip_b.1),
                    fids1[clip_b.2],
                    fids2[clip_b.3],
                    dist,
                    flipped,
                );
                manifold.points.push(contact);
            }
        }
    }
}
