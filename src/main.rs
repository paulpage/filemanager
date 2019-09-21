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

    let mut contents = read_dir(".");
    let mut scroll_idx = 0;
    let mut selected_idx = 0;
    let mut search = String::new();

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit {..} => break 'mainloop,
                // TODO: Replace the following with something good
                // -----------------------------------------------
                Event::KeyDown { keycode: Some(kc), .. } => {
                    match kc {
                        Keycode::Up => {
                            selected_idx = bounded_dec(selected_idx);
                        },
                        Keycode::Down => {
                            selected_idx = bounded_inc(selected_idx, contents.len());
                        },
                        Keycode::PageUp => {
                            scroll_idx = bounded_inc(scroll_idx, contents.len());
                        },
                        Keycode::PageDown => {
                            scroll_idx = bounded_dec(scroll_idx);
                        },
                        Keycode::Escape => {
                            search.clear();
                        },
                        Keycode::Backspace => {
                            std::env::set_current_dir("..").map_err(|e| e.to_string())?;
                            contents = read_dir(".");
                            selected_idx = 0;
                            scroll_idx = 0;
                        },
                        Keycode::Return => {
                            let path =  &contents[selected_idx];
                            if fs::metadata(path).unwrap().is_dir() {
                                std::env::set_current_dir(path).map_err(|e| e.to_string())?;
                                contents = read_dir(".");
                                selected_idx = 0;
                                scroll_idx = 0;
                            } else {
                                println!("TODO: open file");
                            }
                        },
                        _ => {
                            let c = kc.name();
                            if c.len() == 1 {
                                // TODO: Handle shifted keys
                                search.push_str(&c.to_lowercase());
                                for (i, entry) in contents[selected_idx..].iter().enumerate() {
                                    if entry.to_lowercase().starts_with(&search) {
                                        println!("{} == {}", entry.to_lowercase(), &search);
                                        selected_idx =  selected_idx + i;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
                // -----------------------------------------------
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGBA(200, 200, 200, 255));
        canvas.clear();

        // Draw the contents of the current directory
        for (i, entry) in contents[scroll_idx..].iter().enumerate() {
            let surface = font.render(entry)
                .blended(Color::RGBA(40, 0, 0, 255)).map_err(|e| e.to_string())?;
            let texture = texture_creator.create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            let TextureQuery { width, height, .. } = texture.query();
            let padding = 5;
            let target = rect!(0 + padding, i as usize * height as usize, width, height);
            if scroll_idx + i == selected_idx {
                canvas.set_draw_color(Color::RGBA(255, 255, 10, 255));
                canvas.fill_rect(target).map_err(|e| e.to_string())?;
            }
            canvas.copy(&texture, None, Some(target))?;
        }

        canvas.present()
    }

    Ok(())
}
