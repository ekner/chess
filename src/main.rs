extern crate sdl2;

use chess::{Pos, State};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{self, Color};

use sdl2::image::{InitFlag, LoadTexture};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture}; // TextureCreator
use sdl2::video::{Window}; // WindowContext
//use std::alloc::handle_alloc_error;
//use std::env;
use std::path::Path;

//use sdl2::gfx::primitives::DrawRenderer;

const SCREEN_WIDTH: u32 = 1200;
const SCREEN_HEIGHT: u32 = 800;

struct Layout {
    square_size: u32,
    top_left_coord: Pos,
}

fn draw(canvas: &mut Canvas<Window>, textures: &Vec<Texture>, layout: &Layout, state: &State, moving_from: &Option<Pos>) {
    for y in (0..8).rev() {
        for x in 0..8 {
            let mut square_color = if (x + y) % 2 == 1 {
                Color::RGB(255, 206, 158)
            } else {
                Color::RGB(209, 139, 71)
            };

            if let Some(from_pos) = moving_from {
                if from_pos.x == x && from_pos.y == y {
                    square_color = Color::RGB(255, 0,0 );
                }
            }

            canvas.set_draw_color(square_color);

            let x_pos = layout.top_left_coord.x + x * (layout.square_size as i32);
            let y_pos = layout.top_left_coord.y + (7 - y) * (layout.square_size as i32);

            let _res = canvas.fill_rect(Rect::new(x_pos, y_pos, layout.square_size, layout.square_size));

            let piece = state.get(chess::Pos::new(x, y));
            
            match piece {
                None => (),
                Some(piece) => {
                    let index_offset: usize = match piece.player {
                        chess::Player::White => 0,
                        chess::Player::Black => 6
                    };

                    let index = index_offset + (piece.piece_type as usize);
                    let _res = canvas.copy(
                        &textures[index],
                        None, 
                        Some(Rect::new(x_pos, y_pos, layout.square_size, layout.square_size))
                    );
                },
            }
        }
    }

    canvas.present();
}

fn handle_mouse_click(layout: &Layout, state: &mut State, moving_from: &mut Option<Pos>, x: i32, y: i32) {
    //println!("mouse btn down at ({},{})", x, y);

    let x_pos = x / (layout.square_size as i32);
    let y_pos = 7 - y / (layout.square_size as i32);

    //println!("positions: ({},{})", x_pos, y_pos);

    if let Some(pos_from) = moving_from {
        let res = state.move_piece(*pos_from, Pos::new(x_pos, y_pos));

        match res {
            Err(err) => println!("{:?}", err),
            Ok(msg) => println!("{:?}", msg),
        }

        *moving_from = None;
    } else {
        *moving_from = Some(Pos::new(x_pos, y_pos));
    }
}

fn handle_keydown(keycode: Keycode) -> bool {
    if keycode == Keycode::Escape {
        true
    } else if keycode == Keycode::Space {
        println!("space down");
        //for i in 0..400 {
            //canvas.pixel(i as i16, i as i16, 0x0F0000FFu32)?;
        //}
        //canvas.present();
        false
    } else {
        false
    }
}

fn main() -> Result<(), String> {
    let layout = Layout { square_size: 60, top_left_coord: Pos::new(0, 0) };
    
    let sdl_context = sdl2::init()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window(
            "rust-sdl2_gfx: draw line & FPSManager",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut textures = Vec::new();
    
    let list = [
        "white_king", "white_queen", "white_rook", "white_bishop", "white_knight", "white_pawn",
        "black_king", "black_queen", "black_rook", "black_bishop", "black_knight", "black_pawn"
    ];
    
    for name in list {
        let t = texture_creator.load_texture(Path::new("images").join(format!("{}{}", name, ".png")));
        match t {
            Ok(texture) => textures.push(texture),
            Err(err) => {
                println!("Failed to load texture: {}", err);
                std::process::exit(1);
            }
        };
    };

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    let mut state = chess::State::new();
    let mut moving_from: Option<Pos> = None;

    /*assert!(state.move_piece(Pos::new(5, 1), Pos::new(5, 2)).is_ok());
    assert!(state.move_piece(Pos::new(4, 6), Pos::new(4, 4)).is_ok());
    assert!(state.move_piece(Pos::new(6, 1), Pos::new(6, 3)).is_ok());
    assert!(state.move_piece(Pos::new(3, 7), Pos::new(7, 3)).is_ok());*/

    draw(&mut canvas, &textures, &layout, &state, &moving_from);

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if handle_keydown(keycode) {
                        break 'main;
                    }
                }

                Event::MouseButtonDown { x, y, .. } => {
                    handle_mouse_click(&layout, &mut state, &mut moving_from, x, y);
                    draw(&mut canvas, &textures, &layout, &state, &moving_from);
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/*use termion::{color, style};
fn main() {
    println!("I'm using the library");

    let state = chess::State::new();

    println!("{}-----------------", color::Fg(color::White));
    for y in (0..8).rev() {
        for x in 0..8 {
            let piece = state.get(chess::Pos::new(x, y));
            print!("{}", color::Fg(color::White));
            match piece {
                None => print!("|{}#", color::Fg(color::Black)),
                Some(piece) => {
                    match piece.player {
                        chess::Player::White => print!("|{}{}", color::Fg(color::White), piece.piece_type.to_string()),
                        chess::Player::Black => print!("|{}{}", color::Fg(color::Red), piece.piece_type.to_string()),
                    };  
                },
            }
        }
        println!("{}|", color::Fg(color::White));
        println!("-----------------");
    }   
}*/