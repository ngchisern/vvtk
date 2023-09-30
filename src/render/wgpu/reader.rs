use crate::formats::pointxyzrgba::PointXyzRgba;
use crate::formats::PointCloud;
use crate::pcd::read_pcd_file;
use crate::utils::read_file_to_point_cloud;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;

use super::renderable::Renderable;

pub trait RenderReader<T: Renderable> {
    fn start(&mut self) -> Option<T>;
    fn get_at(&mut self, index: usize) -> Option<T>;
    fn get_at_with_lod(&mut self, index: usize, lod: usize) -> Option<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn set_len(&mut self, len: usize);
}

pub struct PcdFileReader {
    files: Vec<Vec<PathBuf>>,
}

impl PcdFileReader {
    pub fn from_directory(directory: &Path) -> Self {
        Self::from_directories(vec![directory])
    }

    pub fn from_directories(directories: Vec<&Path>) -> Self {
        let mut files = vec![];
        for directory in directories {
            let mut files_in_dir = vec![];
            for file_entry in directory.read_dir().unwrap() {
                match file_entry {
                    Ok(entry) => {
                        if let Some(ext) = entry.path().extension() {
                            if ext.eq("pcd") {
                                files_in_dir.push(entry.path());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}")
                    }
                }
            }
            files_in_dir.sort();
            files.push(files_in_dir);
        }
        Self { files }
    }

    pub fn file_at(&self, index: usize) -> Option<&PathBuf> {
        self.files.get(0).and_then(|f| f.get(index))
    }
}

pub struct PointCloudFileReader {
    // files level by level
    files: Vec<Vec<PathBuf>>,
}

impl PointCloudFileReader {
    pub fn from_directory(directory: &Path, file_type: &str) -> Self {
        Self::from_directories(vec![directory], file_type)
    }

    pub fn from_directories(directories: Vec<&Path>, file_type: &str) -> Self {
        let mut files = vec![];

        for directory in directories {
            let mut files_in_dir = vec![];
            for file_entry in directory.read_dir().unwrap() {
                match file_entry {
                    Ok(entry) => {
                        if let Some(ext) = entry.path().extension() {
                            if ext.eq(file_type) {
                                files_in_dir.push(entry.path());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}")
                    }
                }
            }
            files_in_dir.sort();
            files.push(files_in_dir);
        }
        Self { files }
    }
}

impl RenderReader<PointCloud<PointXyzRgba>> for PointCloudFileReader {
    fn start(&mut self) -> Option<PointCloud<PointXyzRgba>> {
        self.get_at_with_lod(0, self.files.len() - 1)
    }

    fn get_at(&mut self, index: usize) -> Option<PointCloud<PointXyzRgba>> {
        self.get_at_with_lod(index, 0)
    }

    fn get_at_with_lod(&mut self, index: usize, lod: usize) -> Option<PointCloud<PointXyzRgba>> {
        let file_path = self.files[lod].get(index)?;
        read_file_to_point_cloud(file_path)
    }

    fn len(&self) -> usize {
        self.files.get(0).map(|f| f.len()).unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.files.get(0).map(|f| f.is_empty()).unwrap_or(true)
    }

    fn set_len(&mut self, _len: usize) {}
}

impl RenderReader<PointCloud<PointXyzRgba>> for PcdFileReader {
    fn start(&mut self) -> Option<PointCloud<PointXyzRgba>> {
        self.get_at_with_lod(0, self.files.len() - 1)
    }

    fn get_at(&mut self, index: usize) -> Option<PointCloud<PointXyzRgba>> {
        self.get_at_with_lod(index, 0)
    }

    fn get_at_with_lod(&mut self, index: usize, lod: usize) -> Option<PointCloud<PointXyzRgba>> {
        self.files
            .get(lod)
            .and_then(|f| f.get(index))
            .and_then(|f| read_pcd_file(f).ok())
            .map(PointCloud::from)
    }

    fn len(&self) -> usize {
        self.files.len()
    }

    fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    fn set_len(&mut self, _len: usize) {}
}

pub struct PcdMemoryReader {
    points: Vec<PointCloud<PointXyzRgba>>,
}

impl PcdMemoryReader {
    pub fn from_vec(points: Vec<PointCloud<PointXyzRgba>>) -> Self {
        Self { points }
    }
}

impl RenderReader<PointCloud<PointXyzRgba>> for PcdMemoryReader {
    fn get_at(&mut self, index: usize) -> Option<PointCloud<PointXyzRgba>> {
        self.points.get(index).cloned()
    }

    fn get_at_with_lod(&mut self, index: usize, _lod: usize) -> Option<PointCloud<PointXyzRgba>> {
        // FIXME: implement lod
        self.get_at(index)
    }

    fn start(&mut self) -> Option<PointCloud<PointXyzRgba>> {
        self.get_at(0)
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn set_len(&mut self, _len: usize) {}
}

#[cfg(feature = "dash")]
pub struct PcdAsyncReader {
    current_frame: u64,
    next_to_get: u64,
    total_frames: u64,
    /// PcdAsyncReader tries to maintain this level of buffer occupancy at any time
    buffer_size: u8,
    buffer: HashMap<FrameRequest, PointCloud<PointXyzRgba>>,
    rx: Receiver<(FrameRequest, PointCloud<PointXyzRgba>)>,
    tx: UnboundedSender<FrameRequest>,
}

#[cfg(feature = "dash")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameRequest {
    pub object_id: u8,
    pub quality: u8,
    pub frame_offset: u64,
}

#[cfg(feature = "dash")]
impl PcdAsyncReader {
    pub fn new(
        rx: Receiver<(FrameRequest, PointCloud<PointXyzRgba>)>,
        tx: UnboundedSender<FrameRequest>,
        buffer_size: Option<u8>,
    ) -> Self {
        let buffer_size = buffer_size.unwrap_or(10);
        Self {
            current_frame: 0,
            next_to_get: 0,
            rx,
            tx,
            buffer_size,
            buffer: HashMap::with_capacity(buffer_size as usize),
            total_frames: 30, // default number of frames
        }
    }

    fn send_next_req(&mut self) {
        println!(
            "next_to_get {}, current_frame {}, buffer_size {}",
            self.next_to_get, self.current_frame, self.buffer_size
        );
        while self.next_to_get - self.current_frame < self.buffer_size as u64 {
            // FIXME: change the object_id and quality.
            self.tx
                .send(FrameRequest {
                    object_id: 0u8,
                    quality: 0u8,
                    frame_offset: self.next_to_get,
                })
                .unwrap();
            self.next_to_get = (self.next_to_get + 1) % (self.len() as u64);
        }
    }
}

#[cfg(feature = "dash")]
impl RenderReader<PointCloud<PointXyzRgba>> for PcdAsyncReader {
    fn start(&mut self) -> Option<PointCloud<PointXyzRgba>> {
        for i in 0..self.buffer_size {
            self.tx
                .send(FrameRequest {
                    object_id: 0,
                    quality: 0,
                    frame_offset: i as u64,
                })
                .unwrap();
        }
        self.next_to_get = self.buffer_size as u64;
        loop {
            if let Some(data) = self.get_at(0) {
                break Some(data);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn get_at(&mut self, index: usize) -> Option<PointCloud<PointXyzRgba>> {
        println!(
            "get_at called with {}. buffer occupancy is {}",
            index,
            self.buffer.len()
        );
        let index = index as u64;

        // remove if we have in the buffer.
        // FIXME: change the object_id and quality.
        if let Some(data) = self.buffer.remove(&FrameRequest {
            object_id: 0u8,
            quality: 0u8,
            frame_offset: index,
        }) {
            println!("get_at returned from buffer ... {}", index);
            self.current_frame = (self.current_frame + 1) % (self.len() as u64);
            self.send_next_req();
            return Some(data);
        }

        loop {
            if let Ok((req, data)) = self.rx.recv() {
                if req.frame_offset == index {
                    println!("get_at returned from channel ... {}", req.frame_offset);
                    self.current_frame = (self.current_frame + 1) % (self.len() as u64);
                    self.send_next_req();
                    return Some(data);
                }

                // enqueues the data into our buffer and preemptively start the next request.
                println!("get_at buffers... {}", req.frame_offset);
                self.buffer.insert(req, data);
            }
        }
    }

    fn get_at_with_lod(&mut self, index: usize, _lod: usize) -> Option<PointCloud<PointXyzRgba>> {
        // FIXME: implement lod
        self.get_at(index)
    }

    fn len(&self) -> usize {
        self.total_frames as usize
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn set_len(&mut self, len: usize) {
        self.total_frames = len as u64;
    }
}

// !! BufRenderReader is not used and comments are deleted.
