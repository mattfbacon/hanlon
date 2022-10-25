use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::CreationContext;
use egui::Sense;
use egui_file::FileDialog;
use epaint::Vec2;

use crate::editor::Editor;
use crate::project::{Component, Project};

enum State {
	Home {
		error: Option<String>,
	},
	Edit {
		file_path: PathBuf,
		project: Arc<Mutex<Project>>,
		editor: Editor,
		_project_stepper_handle: Sender<()>,
	},
}

pub struct App {
	state: State,
	opening: FileDialog,
}

impl App {
	#[must_use]
	pub fn new(_context: &CreationContext<'_>) -> Self {
		Self {
			state: State::Home { error: None },
			opening: FileDialog::open_file(None).filter("han".to_owned()),
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
		match &mut self.state {
			State::Home { error } => {
				egui::CentralPanel::default().show(context, |ui| {
					ui.vertical_centered(|ui| {
						ui.heading("Hanlon");

						if ui.button("New").clicked() {
							todo!("new")
						}

						if ui.button("Open").clicked() {
							self.opening.open();
						}

						if let Some(error) = error {
							ui.colored_label(context.style().visuals.error_fg_color, error);
						}
					});
				});

				if self.opening.show(context).selected() {
					*error = None;
					let file_path = self.opening.path().unwrap();
					match Project::read(&file_path) {
						Ok(project) => {
							let project = Arc::new(Mutex::new(project));
							let (send, recv) = channel();

							std::thread::spawn({
								let project = Arc::clone(&project);
								move || {
									let audio = rodio::OutputStream::try_default().ok();
									let sleep_time = Duration::from_secs_f32(1.0 / 60.0); // 60 fps

									// stop when the sender is dropped
									while let Err(TryRecvError::Empty) = recv.try_recv() {
										let mut project = project.lock().unwrap();
										for sound in project.step_pellets() {
											if let Some((_stream, audio)) = &audio {
												sound.play(audio);
											}
										}
										drop(project);
										std::thread::sleep(sleep_time);
									}
								}
							});

							self.state = State::Edit {
								project,
								file_path,
								editor: Editor::default(),
								_project_stepper_handle: send,
							}
						}
						Err(read_error) => *error = Some(format!("error while reading project: {read_error}")),
					}
				}
			}
			State::Edit {
				editor,
				file_path,
				project,
				_project_stepper_handle: _,
			} => {
				egui::SidePanel::left("palette")
					.default_width(120.0)
					.min_width(120.0)
					.show(context, |ui| {
						egui::ScrollArea::both().show(ui, |ui| {
							ui.vertical_centered(|ui| {
								ui.heading("Palette");
							});

							ui.horizontal_wrapped(|ui| {
								for component in Component::PALETTE_LIST {
									let (rect, response) = ui.allocate_exact_size(
										Vec2::splat(Editor::RELATIVE_COMPONENT_SIZE),
										Sense::drag(),
									);
									if response.drag_started() {
										assert!(editor.dragging.replace(*component).is_none());
									} else if response.drag_released() {
										editor.drag_released_from_outer =
											Some(response.interact_pointer_pos().unwrap());
									}
									component.draw(ui.painter(), rect);
									response.on_hover_text(component.name());
								}
							});
						});
					});

				egui::CentralPanel::default()
					.frame(egui::Frame {
						inner_margin: egui::style::Margin::same(0.0),
						..Default::default()
					})
					.show(context, |ui| {
						editor.show(ui, &mut project.lock().unwrap());
						context.request_repaint();
					});
			}
		}
	}
}
