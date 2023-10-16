use crate::formats::{pointxyzrgba::PointXyzRgba, PointCloud};

// bounding box of the point cloud
pub fn generate_midpoint(points: PointCloud<PointXyzRgba>) -> [f32; 3] {
    let first_point = points.points[0];
    let mut min_x = first_point.x;
    let mut max_x = first_point.x;
    let mut min_y = first_point.y;
    let mut max_y = first_point.y;
    let mut min_z = first_point.z;
    let mut max_z = first_point.z;

    for &point in &points.points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
        min_z = min_z.min(point.z);
        max_z = max_z.max(point.z);
    }

    [
        (max_x - min_x) / 2.0,
        (max_y - min_y) / 2.0,
        (max_z - min_z) / 2.0,
    ]
}
