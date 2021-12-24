extern crate sdl2;

use chess::{Pos, State, MoveSuccess, MoveError, Player, GameStatus};
use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{self, Color};

use sdl2::image::{InitFlag, LoadTexture};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureQuery, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
// TextureCreator
use sdl2::video::{Window, WindowContext}; use std::cell::RefCell;
// WindowContext
//use std::alloc::handle_alloc_error;
//use std::env;
use std::path::Path;
use std::rc::Rc;

//use sdl2::gfx::primitives::DrawRenderer;

const SCREEN_WIDTH: u32 = 1200;
const SCREEN_HEIGHT: u32 = 640;

#[derive(Copy, Clone)]
struct Layout {
    square_size: u32,
    top_left_coord: Pos,
}

impl Layout {
    fn new() -> Layout {
        Layout {
            square_size: 80,
            top_left_coord: Pos::new(0, 0)
        }
    }
}

fn handle_mouse_click(layout: &Layout, state: &mut State, moving_from: &mut Option<Pos>, x: i32, y: i32)
        -> Option<Result<MoveSuccess, MoveError>>
    {
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
        Some(res)
    } else {
        *moving_from = Some(Pos::new(x_pos, y_pos));
        None
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

struct Graphics {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    textures: Vec<Texture>,
    layout: Layout,
}

impl Graphics {
    fn new(canvas: Canvas<Window>, layout: Layout) -> Self {
        let texture_creator = canvas.texture_creator();        

        let mut graphics = Graphics {
            canvas: canvas,
            texture_creator: texture_creator,
            textures: Vec::new(),
            layout: layout,
        };
        
        graphics.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        graphics.canvas.clear();
        graphics.load_textures();
        graphics
    }

    fn load_textures(&mut self) {
        let list = [
            "white_king", "white_queen", "white_rook", "white_bishop", "white_knight", "white_pawn",
            "black_king", "black_queen", "black_rook", "black_bishop", "black_knight", "black_pawn"
        ];
    
        for name in list {
            let t = self.texture_creator.load_texture(Path::new("images").join(format!("{}{}", name, ".png")));
            match t {
                Ok(texture) => self.textures.push(texture),
                Err(err) => {
                    println!("Failed to load texture: {}", err);
                    std::process::exit(1);
                }
            };
        };
    }

    fn draw(&mut self, state: &State, moving_from: &Option<Pos>) {
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
    
                self.canvas.set_draw_color(square_color);
    
                let x_pos = self.layout.top_left_coord.x + x * (self.layout.square_size as i32);
                let y_pos = self.layout.top_left_coord.y + (7 - y) * (self.layout.square_size as i32);
    
                let _res = self.canvas.fill_rect(Rect::new(x_pos, y_pos, self.layout.square_size, self.layout.square_size));
    
                let piece = state.get(Pos::new(x, y));
                
                match piece {
                    None => (),
                    Some(piece) => {
                        let index_offset: usize = match piece.player {
                            Player::White => 0,
                            Player::Black => 6
                        };
    
                        let index = index_offset + (piece.piece_type as usize);
                        let _res = self.canvas.copy(
                            &self.textures[index],
                            None, 
                            Some(Rect::new(x_pos, y_pos, self.layout.square_size, self.layout.square_size))
                        );
                    },
                }
            }
        }
    
        self.canvas.present();
    }

    fn draw_text(&mut self, str: &str, font: &Font, pos: Pos, size: u32, color: Color) {
        let texture_creator = self.canvas.texture_creator();
        let surface = font
            .render(str)
            .blended(color)
            .map_err(|e| e.to_string()).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap();
    
        let TextureQuery { width, height, .. } = texture.query();
    
        let frac = width / height;
        let width = size * frac;
        
        let target = Rect::new(pos.x, pos.y, width, size);
    
        self.canvas.copy(&texture, None, Some(target)).unwrap();
        self.canvas.present();
    
        unsafe {
            texture.destroy();
        }
    }

    fn draw_current_player(&mut self, font: &Font, game_status: GameStatus) {
        let x_pos = self.layout.top_left_coord.x + (self.layout.square_size as i32) * 8 + 5;
        let y_pos = self.layout.top_left_coord.y + 5;
        let str = game_status.to_string();
        self.draw_text(str, font, Pos::new(x_pos, y_pos), 20, Color::RGBA(255, 255, 255, 255));
    }

    fn draw_move_message(&mut self, font: &Font, move_result: Result<MoveSuccess, MoveError>) {
        let x_pos = self.layout.top_left_coord.x + (self.layout.square_size as i32) * 8 + 5;
        let y_pos = self.layout.top_left_coord.y + 40;

        let str = match &move_result {
            Ok(_) => return,
            Err(msg) => msg.to_string(),
        };

        self.draw_text(str, font, Pos::new(x_pos, y_pos), 20, Color::RGBA(255, 0, 0, 255));
    }

    fn draw_info_board(&mut self, font: &Font, move_result: Result<MoveSuccess, MoveError>, game_status: GameStatus) {
        let x_pos = self.layout.top_left_coord.x + (self.layout.square_size as i32) * 8;
        let y_pos = self.layout.top_left_coord.y;
        let width = SCREEN_WIDTH - (x_pos as u32);
        let height = SCREEN_HEIGHT;
    
        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let _r = self.canvas.fill_rect(Rect::new(x_pos, y_pos, width, height));
        
        self.draw_current_player(font, game_status);
        self.draw_move_message(font, move_result);
    }
}

fn create_window() -> Result<(Window, Sdl), String> {
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
    Ok((window, sdl_context))
}

fn main() -> Result<(), String> {
    let layout = Layout::new();
    let (window, sdl_context) = create_window()?;
    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut graphics = Graphics::new(canvas, layout);
    let mut state = State::new();
    let mut moving_from: Option<Pos> = None;

    // font loading
    let font_path = "ubuntu.ttf";
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let mut font = ttf_context.load_font(font_path, 128).unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    /*assert!(state.move_piece(Pos::new(5, 1), Pos::new(5, 2)).is_ok());
    assert!(state.move_piece(Pos::new(4, 6), Pos::new(4, 4)).is_ok());
    assert!(state.move_piece(Pos::new(6, 1), Pos::new(6, 3)).is_ok());
    assert!(state.move_piece(Pos::new(3, 7), Pos::new(7, 3)).is_ok());*/

    graphics.draw(&state, &moving_from);

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
                    let res = handle_mouse_click(&layout, &mut state, &mut moving_from, x, y);
                    graphics.draw(&state, &moving_from);

                    if let Some(res) = res {
                        graphics.draw_info_board(&font, res, state.get_game_status());
                    }
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