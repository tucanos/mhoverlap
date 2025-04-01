use parry3d::{
    math::{Point, Real},
    query::PointQuery,
    shape::TriMesh,
};
pub struct BackgroundMesh {
    shape: TriMesh,
    /// True if a point is on the background mesh
    points_status: bitvec::vec::BitVec,
}

impl BackgroundMesh {
    #[must_use]
    pub fn from_iters<VIT, IIT, VF, IF>(vertices: VF, indices: IF) -> Self
    where
        VIT: Iterator<Item = [f32; 3]>,
        IIT: Iterator<Item = [u32; 3]>,
        VF: Fn() -> VIT,
        IF: Fn() -> IIT,
    {
        let vertices = vertices()
            .map(|pt| Point::new(pt[0], pt[1], pt[2]))
            .collect();
        let indices = indices().collect();
        Self::new(vertices, indices)
    }

    #[must_use]
    pub fn new(vertices: Vec<Point<Real>>, indices: Vec<[u32; 3]>) -> Self {
        let shape = TriMesh::new(vertices, indices).unwrap();
        let points_status = bitvec::vec::BitVec::default();
        Self {
            shape,
            points_status,
        }
    }

    pub fn check_points<IT>(&mut self, points: IT, tolerance: f32)
    where
        IT: Iterator<Item = [f32; 3]>,
    {
        self.points_status.clear();
        if let Some(size) = points.size_hint().1 {
            self.points_status.reserve(size);
        }
        for pt in points {
            let d = self
                .shape
                .distance_to_local_point(&Point::new(pt[0], pt[1], pt[2]), true);
            self.points_status.push(d < tolerance);
        }
    }

    /// Return true if the element is on the background mesh
    pub fn check_element<IT>(&self, element: IT) -> bool
    where
        IT: IntoIterator<Item = usize>,
    {
        for v in element {
            if !self.points_status[v] {
                return false;
            }
        }
        true
    }
    #[must_use]
    pub fn tolerance(&self) -> f32 {
        let min_area = self
            .shape
            .indices()
            .iter()
            .map(|tria| {
                let vs = self.shape.vertices();
                let tria = tria.map(|x| x as usize);
                let v1 = vs[tria[1]] - vs[tria[0]];
                let v2 = vs[tria[2]] - vs[tria[0]];
                v1.cross(&v2).norm_squared()
            })
            .fold(f32::MAX, f32::min)
            .sqrt()
            / 2.;
        // a fraction of the average edge length
        min_area.sqrt() / 10.
    }
}
