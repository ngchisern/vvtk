use clap::Parser;

use crate::{
    midpoint::generate::generate_midpoint,
    pipeline::{channel::Channel, PipelineMessage},
};
use serde_json::json;

use super::Subcommand;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long)]
    des: String,
}

pub struct Midpoint {
    des: String,
    midpoints: Vec<[f32; 3]>,
}

impl Midpoint {
    pub fn from_args(args: Vec<String>) -> Box<dyn Subcommand> {
        let args = Args::parse_from(args);
        Box::new(Midpoint {
            des: args.des,
            midpoints: Vec::new(),
        })
    }
}

impl Subcommand for Midpoint {
    fn handle(&mut self, messages: Vec<PipelineMessage>, channel: &Channel) {
        for message in messages {
            match message {
                PipelineMessage::IndexedPointCloud(pc, _) => {
                    let midpoint = generate_midpoint(pc);
                    self.midpoints.push(midpoint);
                }
                PipelineMessage::Metrics(_) | PipelineMessage::DummyForIncrement => {}
                PipelineMessage::End => {
                    channel.send(message);
                }
            };
        }

        let json = json!({
            "midpoint": self.midpoints,
        });
        let json_str = serde_json::to_string(&json).unwrap();
        std::fs::write(&self.des, json_str).expect("Unable to write file");
    }
}
