//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::egui;
use egui::*;
use tokio::{
    runtime::Runtime,
    sync::{mpsc::Sender, RwLock},
};

mod comm;

const NULL_POS: Pos2 = Pos2 { x: 0f32, y: 0f32 };

pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Arc<RwLock<Vec<Vec<Pos2>>>>,
    stroke: Stroke,
    tx: Sender<Pos2>,
}

fn send(point: Pos2, tx: &mut Sender<Pos2>) {
    match tx.try_send(point) {
        Ok(_) => (),
        Err(e) => println!("Unable to send on mpsc: {e}"),
    }
}
//#[tokio::main]
fn main() {
    let lines = Arc::new(RwLock::new(vec![Default::default()]));

    // Create the runtime
    let rt = Runtime::new().expect("Error creating async runtime");

    // Spawn a future onto the runtime
    let tx = rt.block_on(async { comm::connect(lines.clone()).await });

    let paint = Painting {
        lines: lines,
        stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        tx,
    };

    let options = eframe::NativeOptions::default();
    let app_creator = Box::new(paint);

    eframe::run_native("Sender & Receiver", options, Box::new(|_cc| app_creator)).unwrap();
}

impl eframe::App for Painting {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                response.rect,
            );
            let from_screen = to_screen.inverse();

            let lines = self.lines.blocking_read();

            if painter.ctx().input(|i| i.pointer.primary_down()) {
                let current_line = lines.last().unwrap();

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    //send this to server...
                    let canvas_pos = from_screen * pointer_pos;

                    if current_line.last() != Some(&canvas_pos) {
                        send(canvas_pos, &mut self.tx);
                    }
                } else if !current_line.is_empty() {
                    send(NULL_POS, &mut self.tx);
                }
            };

            let shapes = lines.iter().filter(|line| line.len() >= 2).map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

            painter.extend(shapes);
        });
    }
}
