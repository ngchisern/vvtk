use crate::formats::{pointxyzrgba::PointXyzRgba, PointCloud};
use crate::subsample::random_sampler::subsample;
use crate::utils::get_pc_bound;

pub fn lodify(
    points: &PointCloud<PointXyzRgba>,
    partitions: (usize, usize, usize),
    proportions: Vec<usize>,
    points_per_voxel_threshold: usize,
) -> (
    PointCloud<PointXyzRgba>,
    Vec<PointCloud<PointXyzRgba>>,
    Vec<Vec<usize>>,
) {
    if points.points.is_empty() {
        return (points.clone(), vec![], vec![]);
    } else {
        let points = subsample(points, proportions, points_per_voxel_threshold);

        if points.len() == 1 {
            return (points[0].clone(), vec![], vec![]);
        }

        let base_pc = points[0].clone();
        let additional_pcs = points[1..].to_vec();

        let (pc_by_segment, point_num_by_resolution) =
            Partitioner::new(additional_pcs, partitions).get_additional_points_by_segment();

        (base_pc, pc_by_segment, point_num_by_resolution)
    }
}

// Partitioner for partitioning additional points by segments
// each segment will contain points of different resolution
// e.g. (r0, r0, ...., r0, r1, r1, ...., r1, r2, r2, ...., r2)
struct Partitioner {
    point_clouds: Vec<PointCloud<PointXyzRgba>>,
    partitions: (usize, usize, usize),
}

impl Partitioner {
    pub fn new(
        point_clouds: Vec<PointCloud<PointXyzRgba>>,
        partitions: (usize, usize, usize),
    ) -> Self {
        Partitioner {
            point_clouds,
            partitions,
        }
    }

    pub fn get_additional_points_by_segment(
        &self,
    ) -> (Vec<PointCloud<PointXyzRgba>>, Vec<Vec<usize>>) {
        let mut partitioned_pcs = vec![];

        for pc in &self.point_clouds {
            let partitioned_pc = partition(&pc, self.partitions);
            partitioned_pcs.push(partitioned_pc);
        }

        let num_of_segments = self.partitions.0 * self.partitions.1 * self.partitions.2;
        let mut points_by_segments = vec![vec![]; num_of_segments];
        let mut point_num_by_resolutions = vec![vec![]; num_of_segments];

        for pc in &partitioned_pcs {
            for (i, segment) in pc.segments.iter().enumerate() {
                points_by_segments[i].extend(segment.points.iter().cloned());
                point_num_by_resolutions[i].push(segment.points.len());
            }
        }

        let mut new_pcs = vec![];

        for points in points_by_segments {
            new_pcs.push(PointCloud::new(points.len(), points));
        }

        (new_pcs, point_num_by_resolutions)
    }
}

pub fn partition(
    pc: &PointCloud<PointXyzRgba>,
    partitions: (usize, usize, usize),
) -> PointCloud<PointXyzRgba> {
    let bound = get_pc_bound(&pc);
    let mut partitioned_points = vec![vec![]; partitions.0 * partitions.1 * partitions.2];

    for point in &pc.points {
        let index = bound.get_bound_index(point, partitions);
        partitioned_points[index].push(*point);
    }

    PointCloud::new_with_segments(partitioned_points)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]

    fn test_partition() {
        let points = vec![
            PointXyzRgba {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            PointXyzRgba {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            PointXyzRgba {
                x: 2.0,
                y: 2.0,
                z: 2.0,
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            PointXyzRgba {
                x: 3.0,
                y: 3.0,
                z: 3.0,
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
        ];

        let pc = PointCloud::new(4, points);

        let result = partition(&pc, (2, 2, 2));

        assert_eq!(result.points.len(), 4);
        assert_eq!(result.segments.len(), 8);
        assert_eq!(result.segments[0].points.len(), 2);
        assert_eq!(result.segments[1].points.len(), 0);
        assert_eq!(result.segments[2].points.len(), 0);
        assert_eq!(result.segments[3].points.len(), 0);
        assert_eq!(result.segments[4].points.len(), 0);
        assert_eq!(result.segments[5].points.len(), 0);
        assert_eq!(result.segments[6].points.len(), 0);
        assert_eq!(result.segments[7].points.len(), 2);
    }
}
