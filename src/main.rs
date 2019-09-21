extern crate sdl2;

use std::path::PathBuf;
use std::fs;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::pixels::Color;

static SCREEN_WIDTH : u32 = 800;
static SCREEN_HEIGHT : u32 = 600;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn bounded_dec(target: usize) -> usize {
    if target > 0 { target - 1 } else { target }
}

fn bounded_inc(target: usize, upper: usize) -> usize {
    if target < upper { target + 1 } else { target }
}

fn read_dir<T: Into<PathBuf>>(path: T) -> Vec<String> {
    fs::read_dir(&path.into())
        .unwrap()
        .map(|result| result.map(|entry| entry.path().display().to_string()).unwrap())
        .map(|s| s.replacen("./", "", 1))
        .collect()
}

struct App {
    contents: Vec<String>,
    scroll_idx: usize,
    selected_idx: usize,
    search: String,
}

impl App {
    fn set_dir<T: std::convert::AsRef<std::path::Path>>(&mut self, path: T) {
        std::env::set_current_dir(path).unwrap();
        self.contents = read_dir(".");
        self.scroll_idx = 0;
        self.selected_idx = 0;
        self.search.clear();
    }
}

fn main() -> Result<(), String> {
    // Initialize video subsystem
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let window = video_subsys.window("SDL2_TTF Example", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Load resources
    let font_path = "data/LiberationSans-Regular.ttf";
    let font = ttf_context.load_font(font_path, 16)?;

    let mut app = App {
        contents: read_dir("."),
        scroll_idx: 0,
        selected_idx: 0,
        search: String::new(),
    };

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit {..} => break 'mainloop,
                Event::KeyDown { keycode: Some(kc), .. } => {
                    match kc {
                        Keycode::Up => {
                            app.selected_idx = bounded_dec(app.selected_idx);
                            app.search.clear();
                        },
                        Keycode::Down => {
                            app.selected_idx = bounded_inc(app.selected_idx, app.contents.len());
                            app.search.clear();
                        },
                        Keycode::PageUp => {
                            app.scroll_idx = bounded_inc(app.scroll_idx, app.contents.len());
                        },
                        Keycode::PageDown => {
                            app.scroll_idx = bounded_dec(app.scroll_idx);
                        },
                        Keycode::Escape => {
                            app.search.clear();
                        },
                        Keycode::Backspace => {
                            app.set_dir("..");
                        },
                        Keycode::Return => {
                            let path =  &app.contents[app.selected_idx].clone();
                            if fs::metadata(path).unwrap().is_dir() {
                                app.set_dir(path);
                            } else {
                                println!("TODO: open file");
                            }
                        },
                        _ => {
                            let c = kc.name();
                            if c.len() == 1 {
                                // TODO: Handle shifted keys
                                app.search.push_str(&c.to_lowercase());
                                for (i, entry) in app.contents[app.selected_idx..].iter().enumerate() {
                                    if entry.to_lowercase().starts_with(&app.search) {
                                        println!("{} == {}", entry.to_lowercase(), &app.search);
                                        app.selected_idx =  app.selected_idx + i;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGBA(200, 200, 200, 255));
        canvas.clear();

        // Draw the app.contents of the current directory
        for (i, entry) in app.contents[app.scroll_idx..].iter().enumerate() {
            let surface = font.render(entry)
                .blended(Color::RGBA(40, 0, 0, 255)).map_err(|e| e.to_string())?;
            let texture = texture_creator.create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            let TextureQuery { width, height, .. } = texture.query();
            let padding = 5;
            let target = rect!(0 + padding, i as usize * height as usize, width, height);
            if app.scroll_idx + i == app.selected_idx {
                canvas.set_draw_color(Color::RGBA(255, 255, 10, 255));
                canvas.fill_rect(target).map_err(|e| e.to_string())?;
            }
            canvas.copy(&texture, None, Some(target))?;
        }

        canvas.present()
    }

    Ok(())
}
