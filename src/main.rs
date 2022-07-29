#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
extern crate eframe;
extern crate lopdf;
extern crate rfd;

mod split;

use eframe::{
    egui::{
        CentralPanel, Context, Layout, RichText, ScrollArea, TextEdit, TopBottomPanel, Vec2,
        Visuals,
    },
    epaint::Color32,
    run_native, App, Frame, NativeOptions,
};
use rfd::FileDialog;
use split::{Split, PDF};

const SPACE: f32 = 5.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrMessages {
    NoNil,
    NoInput,
    NoNumber,
    NoFile,
    FileSave,
    FileLoad,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
struct SplitParams {
    params: Vec<SplitUnit>,
    file: String,
    errors: String,
    done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SplitUnit {
    from: String,
    to: String,
    filename: String,
}

impl App for SplitParams {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF SPLIT");
            ui.separator();
            ui.add_space(SPACE);
            ui.horizontal(|ui| {
                ui.add_sized(Vec2::new(180., 20.), TextEdit::singleline(&mut self.file));

                if ui.button("File").clicked() {
                    let file = FileDialog::new().add_filter("pdf", &["pdf"]).pick_file();
                    self.file = file.unwrap().display().to_string();
                }
            });
            ui.add_space(SPACE);
            ui.separator();
            if !self.errors.is_empty() {
                ui.add_space(SPACE);
                ui.label(
                    RichText::new(Box::leak(self.errors.clone().into_boxed_str()))
                        .color(Color32::RED)
                );
                ui.add_space(SPACE);
            }
            if self.done {
                ui.add_space(SPACE);
                ui.label(
                    RichText::new("Job done!")
                        .color(Color32::GREEN)
                );
                ui.add_space(SPACE);
            }
            ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("From");
                    ui.add_space(21.);
                    ui.add_sized(
                        Vec2::new(80., 20.),
                        TextEdit::singleline(&mut self.params[0].from),
                    );
                    ui.label("To");
                    ui.add_sized(
                        Vec2::new(80., 20.),
                        TextEdit::singleline(&mut self.params[0].to),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Filename");
                    ui.add_sized(
                        Vec2::new(190., 20.),
                        TextEdit::singleline(&mut self.params[0].filename),
                    );
                });

                for a in self.params[1..].iter_mut() {
                    ui.add_space(SPACE);
                    ui.horizontal(|ui| {
                        ui.label("From");
                        ui.add_space(21.);
                        ui.add_sized(Vec2::new(80., 20.), TextEdit::singleline(&mut a.from));
                        ui.label("To");
                        ui.add_sized(Vec2::new(80., 20.), TextEdit::singleline(&mut a.to));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Filename");
                        ui.add_sized(Vec2::new(190., 20.), TextEdit::singleline(&mut a.filename));
                    });
                }

                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        self.params.push(SplitUnit::empty());
                    }
                    if self.params.len() > 1 {
                        if ui.button("Delete Last").clicked() {
                            let cut = self.params.len() - 1;
                            self.params.truncate(cut);
                        }
                    }
                });
            });
            ui.separator();
                ui.add_space(SPACE);
        });
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(30.);
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.button(RichText::new("Do Split").size(20.)).clicked() {
                        if self.file.is_empty() {
                            self.set_error(ErrMessages::NoFile);
                        } else {
                            self.set_error(ErrMessages::Empty);

                            let file_path_as_str = Box::leak(self.file.clone().into_boxed_str());
                            match convert_splits(self.params.clone()) {
                                Ok(pars) => match PDF::new(&file_path_as_str, pars) {
                                    Ok(mut pdf) => {
                                        self.set_error(ErrMessages::Empty);
                                        match pdf.split() {
                                            Ok(_) => {
                                                self.done = true;        
                                                ()},
                                            Err(err) => self.set_error(err),
                                        }
                                    }
                                    Err(_) => self.set_error(ErrMessages::FileLoad),
                                },
                                Err(err) => self.set_error(err),
                            }
                        }
                    }
                    if ui.button(RichText::new("Reset").size(20.)).clicked() {
                        self.file = String::from("");
                        self.params = vec![SplitUnit::empty()];
                        self.errors = String::new();
                        self.done = false;
                    }
                });
                ui.add_space(SPACE*2.);
            });

        });
    }
}

impl SplitParams {
    fn new() -> SplitParams {
        SplitParams {
            params: vec![SplitUnit::empty()],
            file: String::from(""),
            errors: String::new(),
            done: false,
        }
    }

    fn set_error(&mut self, err: ErrMessages) {
        match err {
            ErrMessages::NoNil => self.errors = String::from("No zero allowed."),
            ErrMessages::NoNumber => self.errors = String::from("Provided input is not a number."),
            ErrMessages::NoFile => self.errors = String::from("No file path provided."),
            ErrMessages::NoInput => self.errors = String::from("One or more inputs are empty"),
            ErrMessages::FileSave => self.errors = String::from("Failed to save files."),
            ErrMessages::FileLoad => self.errors = String::from("Unabled to load file."),
            ErrMessages::Empty => self.errors = String::new(),
        }
    }
}

impl SplitUnit {
    fn empty() -> SplitUnit {
        SplitUnit {
            from: String::from(""),
            to: String::from(""),
            filename: String::from(""),
        }
    }
}

fn convert_splits<'a>(splits: Vec<SplitUnit>) -> Result<Vec<Split<'a>>, ErrMessages> {
    let mut collected: Vec<Split<'a>> = Vec::new();
    for split in splits {
        if split.to.is_empty() || split.from.is_empty() || split.filename.is_empty() {
            return Err(ErrMessages::NoInput);
        }

        let mut new_split = Split::new();
        match split.from.parse() {
            Ok(v) => {
                if v == 0 {
                    return Err(ErrMessages::NoNil);
                } else {
                    new_split.0 = v;
                }
            }
            Err(_) => return Err(ErrMessages::NoNumber),
        };
        match split.to.parse() {
            Ok(v) => {
                if v == 0 {
                    return Err(ErrMessages::NoNil);
                } else {
                    new_split.1 = v;
                }
            }
            Err(_) => return Err(ErrMessages::NoNumber),
        };
        new_split.2 = Box::leak(split.filename.into_boxed_str());
        collected.push(new_split);
    }
    return Ok(collected);
}

fn main() {
    let app = SplitParams::new();
    let mut window_options = NativeOptions::default();
    window_options.initial_window_size = Some(Vec2::new(285., 395.));
    run_native("Split Pdf", window_options, Box::new(|_cc| Box::new(app)));
}
