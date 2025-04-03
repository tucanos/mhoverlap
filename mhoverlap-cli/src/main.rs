use core::f32;

use clap::Parser;
use mhoverlap::BackgroundMesh;
use qd_cgns_rs::{BCType, ElementType, PointSetType, SectionInfo, cgsize};

/// CLI application for working with CGNS files
#[derive(Parser, Debug)]
#[command(about = "Tag elements of a CGNS file which overlap with an other CGNS file")]
struct Cli {
    /// Reference CGNS file
    reference: String,

    /// CGNS file to check
    check: String,

    /// Tolerance value. Automatically computed if not specified.
    #[arg(short, long)]
    tolerance: Option<f32>,

    /// Result CGNS file
    result: String,
}

struct ZoneIterator {
    file: qd_cgns_rs::File,
    base: qd_cgns_rs::Base,
    zone: qd_cgns_rs::Zone,
    num_vertices: usize,
    num_cells: usize,
}

impl ZoneIterator {
    const COORD_NAMES: [&str; 3] = ["CoordinateX", "CoordinateY", "CoordinateZ"];
    pub fn new(filename: &str) -> Self {
        let file = qd_cgns_rs::open(filename, qd_cgns_rs::Mode::Read).expect("Cannot open file");
        let base = 1.into();
        let zone = 1.into();
        let (_, sizes) = file.zone_read(base, zone).unwrap();
        let num_vertices = sizes[0];
        let (section_info, _) = file
            .section_read(base, zone, 1)
            .expect("Cannot read section");
        assert_eq!(section_info.typ, ElementType::TRI_3);
        let num_cells = section_info.end - section_info.start + 1;
        Self {
            file,
            base,
            zone,
            num_vertices,
            num_cells,
        }
    }
    fn read_coords(&self, index: usize) -> Vec<f64> {
        let mut buff = vec![0.; self.num_vertices];
        let label = Self::COORD_NAMES[index];
        self.file
            .coord_read(self.base, self.zone, label, 1, self.num_vertices, &mut buff)
            .expect("Cannot read coordinates");
        buff
    }
    pub fn vertices(&self) -> impl ExactSizeIterator<Item = [f32; 3]> {
        self.read_coords(0)
            .into_iter()
            .zip(self.read_coords(1))
            .zip(self.read_coords(2))
            .map(|((x, y), z)| [x as f32, y as f32, z as f32])
    }

    fn elements(&self) -> impl ExactSizeIterator<Item = [u32; 3]> {
        let mut cells = vec![0; 3 * self.num_cells];
        self.file
            .elements_read(self.base, self.zone, 1, &mut cells, &mut [])
            .unwrap();
        let mut offset = 0;
        (0..self.num_cells).map(move |_| {
            let r = std::array::from_fn(|i| (cells[offset + i] - 1) as u32);
            offset += 3;
            r
        })
    }

    pub fn save(&self, filename: &str, overlap: &[cgsize], nooverlap: &[cgsize]) {
        let mut f = qd_cgns_rs::open(filename, qd_cgns_rs::Mode::Write).unwrap();
        let base = f.base_write("Base", 3, 3).unwrap();
        let zone_cg_id = f
            .zone_write(base, "Zone", self.num_vertices, self.num_cells, 0)
            .unwrap();
        for (i, label) in Self::COORD_NAMES.iter().enumerate() {
            f.coord_write(base, zone_cg_id, label, &self.read_coords(i))
                .unwrap();
        }
        let mut cells = vec![0; 3 * self.num_cells];
        self.file
            .elements_read(self.base, self.zone, 1, &mut cells, &mut [])
            .unwrap();
        f.section_write(
            base,
            zone_cg_id,
            &SectionInfo::new(ElementType::TRI_3, self.num_cells),
            &cells,
        )
        .unwrap();
        f.boco_write(
            base,
            zone_cg_id,
            "overlap",
            BCType::BCWall,
            PointSetType::PointList,
            overlap,
        )
        .unwrap();
        f.boco_write(
            base,
            zone_cg_id,
            "nooverlap",
            BCType::BCWall,
            PointSetType::PointList,
            nooverlap,
        )
        .unwrap();
    }
}

fn main() {
    let cli = Cli::parse();
    let background = ZoneIterator::new(&cli.reference);
    println!("Creating backgroud BVH");
    let mut background_mesh =
        BackgroundMesh::from_iters(|| background.vertices(), || background.elements());
    let checked = ZoneIterator::new(&cli.check);
    let tolerance = cli.tolerance.unwrap_or_else(|| background_mesh.tolerance());
    println!("Tolerance: {tolerance}");
    println!("Projecting points");
    background_mesh.check_points(checked.vertices(), tolerance);
    println!("Checking triangles overlapping");
    let mut overlap = Vec::with_capacity(checked.num_cells);
    let mut nooverlap = Vec::with_capacity(checked.num_cells);
    for (element_id, element) in checked.elements().enumerate() {
        let element_id = element_id as cgsize + 1;
        if background_mesh.check_element(element.map(|x| x as usize)) {
            overlap.push(element_id);
        } else {
            nooverlap.push(element_id);
        }
    }
    println!(
        "Number of checked triangles: {}",
        overlap.len() + nooverlap.len()
    );
    println!(
        "Number of triangles overlapping background mesh: {}",
        overlap.len()
    );
    println!(
        "Number of triangles not overlapping background mesh: {}",
        nooverlap.len()
    );
    checked.save(&cli.result, &overlap, &nooverlap);
}
