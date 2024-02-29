use crate::formats::metadata::MetaData;
use crate::formats::pointxyzrgba::PointXyzRgba;

use super::antialias::AntiAlias;
use super::camera::CameraState;

use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::vec::Vec;

pub struct ResolutionController {
    anchor_spacing: f32,
    anti_alias: AntiAlias,
    metadata: Option<MetaData>,
}

impl ResolutionController {
    pub fn new(
        points: &Vec<PointXyzRgba>,
        metadata: Option<MetaData>,
        anti_alias: AntiAlias,
    ) -> Self {
        let points = anti_alias.apply(points);
        let anchor_spacing = Self::calculate_spacing(&points);

        Self {
            anchor_spacing,
            anti_alias,
            metadata,
        }
    }

    pub fn get_desired_num_points(
        &self,
        index: usize,
        camera_state: &CameraState,
        exclude_base_points: bool,
    ) -> Vec<usize> {
        let metadata = self.metadata.as_ref().unwrap();

        // let centroids = metadata.centroids.get(index).unwrap();
        let bounds = metadata
            .bounds
            .get(index)
            .unwrap()
            .partition(metadata.partitions);
        let base_point_num = metadata.base_point_num.get(index).unwrap();
        let additional_point_nums = metadata.additional_point_nums.get(index).unwrap();

        let mut desired_num_points = vec![0; bounds.len()];

        for i in 0..bounds.len() {
            let bound = bounds.get(i).unwrap();
            let point_num = base_point_num.get(i).unwrap();
            let additional_point_num = additional_point_nums.get(i).unwrap();

            let margin = (bound.max_x - bound.min_x)
                .max(bound.max_y - bound.min_y)
                .max(bound.max_z - bound.min_z)
                / (self.anti_alias.scale * 2.0);

            let z = (bound
                .get_vertexes()
                .iter()
                .map(|poi| camera_state.distance(self.anti_alias.apply_single(poi)))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                - margin)
                .max(0.);

            let desired_num = self.get_desired_num_points_at(camera_state, z, *point_num);
            let deficit = desired_num - (*point_num).min(desired_num);
            let additional_point_needed = self.binary_search(additional_point_num, deficit);

            desired_num_points[i] = if exclude_base_points {
                additional_point_needed
            } else {
                *point_num + additional_point_needed
            };
        }

        return desired_num_points;
    }

    /// Use binary search to get the minimum additional points needed to achieve the desired points
    fn binary_search(&self, additional_point_nums: &Vec<usize>, desired_point_num: usize) -> usize {
        let mut left = 0;
        let mut right = additional_point_nums.len() - 1;

        while left < right {
            let mid = (left + right) / 2;
            let mid_val = additional_point_nums.get(mid).unwrap();

            if *mid_val < desired_point_num {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        return additional_point_nums.get(left).unwrap().clone();
    }

    fn get_desired_num_points_at(
        &self,
        camera_state: &CameraState,
        z: f32,
        current_point_num: usize,
    ) -> usize {
        let window_size = camera_state.get_window_size();
        let (width, height) = camera_state.get_plane_at_z(z);
        // println!("z: {}, width: {}, height: {}", z, width, height);
        // println!("window_size: {:?}", window_size);

        let x_spacing = width / window_size.width as f32;
        let y_spacing = height / window_size.height as f32;
        // println!("x_spacing: {}, y_spacing: {}", x_spacing, y_spacing);

        let desired_spacing = x_spacing.min(y_spacing);
        let scaling_factor = (self.anchor_spacing / desired_spacing).powi(3);
        // let scaling_factor = self.anchor_spacing / desired_spacing;
        // println!(
        //     "desired_spacing: {}, anchor_spacing: {}, scaling_factor: {}",
        //     desired_spacing, self.anchor_spacing, scaling_factor
        // );

        return (current_point_num as f32 * scaling_factor as f32) as usize;
    }

    fn calculate_spacing(points: &Vec<PointXyzRgba>) -> f32 {
        let mut tree = KdTree::new(3);
        for (i, p) in points.iter().enumerate() {
            tree.add([p.x, p.y, p.z], i).unwrap();
        }

        let mut sum = 0.0;
        let k_nearest = 4;

        for p in points.iter() {
            let avg_spacing = tree
                .nearest(&[p.x, p.y, p.z], k_nearest, &squared_euclidean)
                .unwrap()
                .iter()
                .skip(1) // ignore the first point (same point)
                .map(|(d, _)| d.sqrt())
                .sum::<f32>()
                / (k_nearest - 1) as f32;

            sum += avg_spacing;
        }

        sum / points.len() as f32
    }
}
