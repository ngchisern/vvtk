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
        (max_x + min_x) / 2.0,
        (max_y + min_y) / 2.0,
        (max_z + min_z) / 2.0,
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::read_file_to_point_cloud;

    use std::path::PathBuf;
    #[test]
    fn test_midpoint() {
        let pcd_path = PathBuf::from("./test_files/pcd_ascii/longdress_vox10_1213_short.pcd");
        let pcd = read_file_to_point_cloud(&pcd_path).unwrap();

        let midpoint = generate_midpoint(pcd);

        let meta_file =
            PathBuf::from("./test_files/pcd_ascii/longdress_vox10_1213_short_meta.json");
        let meta_json = std::fs::read_to_string(&meta_file).unwrap();
        let meta_json: serde_json::Value = serde_json::from_str(&meta_json).unwrap();

        let expected_midpoint = meta_json["midpoint"].as_array().unwrap()[0]
            .as_array()
            .unwrap();

        assert_eq!(midpoint[0], expected_midpoint[0]);
        assert_eq!(midpoint[1], expected_midpoint[1]);
        assert_eq!(midpoint[2], expected_midpoint[2]);
    }
}
