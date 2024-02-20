use serde::{Deserialize, Serialize};

use super::bounds::Bounds;

#[derive(Serialize, Deserialize)]
pub struct MetaData {
    pub bounds: Vec<Bounds>,
    pub base_point_num: Vec<Vec<usize>>,
    pub additional_point_nums: Vec<Vec<Vec<usize>>>,
    pub num_of_additional_file: usize,
    pub partitions: (usize, usize, usize),
}

impl MetaData {
    pub fn new(
        bounds: Vec<Bounds>,
        base_point_nums: Vec<Vec<usize>>,
        additional_point_nums: Vec<Vec<Vec<usize>>>,
        num_of_additional_file: usize,
        partitions: (usize, usize, usize),
    ) -> Self {
        Self {
            bounds,
            base_point_num: base_point_nums,
            additional_point_nums,
            num_of_additional_file,
            partitions,
        }
    }

    pub fn default() -> Self {
        Self {
            bounds: vec![],
            base_point_num: vec![],
            additional_point_nums: vec![],
            num_of_additional_file: 0,
            partitions: (0, 0, 0),
        }
    }

    pub fn next(
        &mut self,
        bound: Bounds,
        base_point_num: Vec<usize>,
        additional_point_num: Vec<Vec<usize>>,
    ) {
        self.bounds.push(bound);
        self.base_point_num.push(base_point_num);
        self.additional_point_nums.push(additional_point_num);
    }
}
