use crate::spotify;
use crate::spotify::TrackInfo;
use crate::state::{CodeStyle, DecodedImage};
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};

pub enum WorkerCommand {
    FetchTrack {
        request_id: u64,
        original_url: String,
    },
    FetchCode {
        request_id: u64,
        track_id: String,
        style: CodeStyle,
    },
}

pub enum WorkerResult {
    Track {
        request_id: u64,
        result: Result<TrackInfo, String>,
    },
    Code {
        request_id: u64,
        track_id: String,
        style: CodeStyle,
        result: Result<DecodedImage, String>,
    },
}

pub fn spawn(ctx: egui::Context) -> (Sender<WorkerCommand>, Receiver<WorkerResult>) {
    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for command in command_rx {
            match command {
                WorkerCommand::FetchTrack {
                    request_id,
                    original_url,
                } => {
                    let result = spotify::fetch_track(&original_url);
                    let _ = result_tx.send(WorkerResult::Track { request_id, result });
                }
                WorkerCommand::FetchCode {
                    request_id,
                    track_id,
                    style,
                } => {
                    let result = spotify::fetch_code_image(&track_id, style);
                    let _ = result_tx.send(WorkerResult::Code {
                        request_id,
                        track_id,
                        style,
                        result,
                    });
                }
            }
            ctx.request_repaint();
        }
    });
    (command_tx, result_rx)
}
