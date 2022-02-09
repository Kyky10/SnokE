#![no_main]

use core::time;
use std::{alloc::System, thread, sync::Mutex};

#[global_allocator]
static A: System = System;
use crossterm::{InputEvent, KeyEvent, TerminalCursor, Color, AsyncReader};
use lazy_static::lazy_static;
use crossterm::style;
use rand::Rng;
use winconsole::console::{self};
use std::io::{self, Write};

lazy_static!{
    static ref CROSSTERM: crossterm::Crossterm = crossterm::Crossterm::new();   
    static ref CURSOR:TerminalCursor = crossterm::cursor();
    static ref STDIN:Mutex<AsyncReader> = Mutex::new(CROSSTERM.input().read_async());
}

const PLAYABLE_SIZE_MAX: u16 = 40;
const GAME_VERSION: &str = "1.1";
static mut GAME_PAUSED: bool = false;
static mut SPEED_MULT: u64 = SPEED_MULT_ORIGINAL;
const SPEED_MULT_ORIGINAL: u64 = 6;


#[no_mangle]
pub fn main(_argc: i32, _argv: *const *const u8) -> i32 {    
    console::set_window_size(PLAYABLE_SIZE_MAX * 2, PLAYABLE_SIZE_MAX).unwrap();
    CURSOR.hide().unwrap();
    
    loop{
        unsafe{ SPEED_MULT = SPEED_MULT_ORIGINAL };
        crossterm::terminal().clear(crossterm::ClearType::All).unwrap();        
        game_loop()
    }
    
    0
}


fn next_key() -> Option<InputEvent> {
    return STDIN.lock().unwrap().next();
}

fn game_loop(){
    let mut apple_pos: Point = reset_apple();
    let mut points: i32 = 0;
    let mut snake = &mut Snake { 
        direction: Direction::Right, 
        head_pos: Point { x: 2, y: 2 }, 
        tail: Vec::from([Point { x: 2, y: 0 }, Point { x: 2, y: 1 }]),
    };

    loop {
        let mut new_direction = Direction::None;
        
        // Read input (if any)
        let input = next_key();

        // If a key was pressed
       if let Some(b) = input {
           let snake_direction = snake.direction;
           match b {
               InputEvent::Keyboard(event) => 
               {      
                   match event {
                    KeyEvent::Char(ch) => {
                           match ch {
                               'q' => break,
                               'c' => break,
                               '-' => unsafe{ if SPEED_MULT < 20 { SPEED_MULT += 1; continue }},
                               '+' => unsafe{ if SPEED_MULT != 1 { SPEED_MULT -= 1; continue }},
                               ' ' => unsafe{ GAME_PAUSED = !GAME_PAUSED; continue },
                               _ => {
                                    continue;
                               }
                           }
                       }
                       
                    KeyEvent::Down => {
                           if snake_direction != Direction::Up && snake_direction != Direction::Down {
                               new_direction = Direction::Down;
                           }else{
                               continue;
                           }
                       }
                       
                    KeyEvent::Up => {
                           if snake_direction != Direction::Down && snake_direction != Direction::Up {
                               new_direction = Direction::Up;
                           }else{
                               continue;
                           }
                       }
                       
                    KeyEvent::Left => {
                           if snake_direction != Direction::Right && snake_direction != Direction::Left {
                               new_direction = Direction::Left;
                           }else{
                               continue;
                           }
                       }
                       
                    KeyEvent::Right => {
                           if snake_direction != Direction::Left && snake_direction != Direction::Right {
                               new_direction = Direction::Right;
                           }else{
                               continue;
                           }
                       },
                    _ => (),
                }
               },
               _ => {}
           }
        }
        
        if unsafe{ GAME_PAUSED } {
            draw_game(snake, points, apple_pos);
            sleep(unsafe{ SPEED_MULT } * 20);
            continue;
        }
        
        if new_direction != Direction::None {
            (*snake).direction = new_direction;
        }
        
        // Move the snake
        move_snake(snake);
        
        if !check_snake_out_pos(snake) {
            game_over(points);
            while next_key() != Some(InputEvent::Keyboard(KeyEvent::Char('\n'))){
                sleep(10)
            }
            return;
        }
        
        if snake_eated_apple(snake, apple_pos) {
            points += 1;
            snake.tail.push(Point { x: snake.head_pos.x, y: snake.head_pos.y });
            apple_pos = reset_apple();
        }
        
        draw_game(snake, points, apple_pos);
        
        sleep(unsafe{ SPEED_MULT } * 20);
    }
}


fn reset_apple() -> Point {
    let mut rand = rand::thread_rng();
    Point{
        x: rand.gen_range(0..PLAYABLE_SIZE_MAX) as i16, 
        y: rand.gen_range(1..PLAYABLE_SIZE_MAX) as i16
    }
}


fn game_over(points: i32) {
    let term = crossterm::terminal();
    term.clear(crossterm::ClearType::All).unwrap();    
    println!("You died and won {} points!", points);
    println!("Ctrl + C to exit or Enter to retry");
}

fn check_snake_out_pos(snake: &Snake) -> bool{
    let head_pos = snake.head_pos;
    if head_pos.x < 0 || head_pos.x > PLAYABLE_SIZE_MAX as i16 || head_pos.y < 1 || head_pos.y > PLAYABLE_SIZE_MAX as i16 {
        return false
    }
    
    
    for part in snake.tail.iter(){
        if head_pos.x == part.x && head_pos.y == part.y {
            return false
        }
    }
    
    return true
}

fn snake_eated_apple(snake: &Snake, apple: Point) -> bool{
    snake.head_pos.x == apple.x && snake.head_pos.y == apple.y
}


fn move_snake(snake: &mut Snake){
    let prev_snake_pos = snake.head_pos;
    match snake.direction {
        Direction::Up => {
            snake.head_pos.y -= 1;
        }
        Direction::Down => {
            snake.head_pos.y += 1;
        }
        Direction::Left => {
            snake.head_pos.x -= 1;
        }
        Direction::Right => {
            snake.head_pos.x += 1;
        }
        _ => {}
    }
    
    snake.tail.rotate_left(1);
    
    for part in snake.tail.iter() {
        cmd_goto(part.x * 2, part.y);
        printc("  ", Color::Black, Color::Black);
    }
    
    
    snake.tail.remove(snake.tail.len() - 1);
    snake.tail.push(prev_snake_pos);
    
    for part in snake.tail.iter() {
        cmd_goto(part.x * 2, part.y);
        printc("  ", Color::Black, Color::Black);
    }
}


// const left_down: &str = "╗";
// const right_down: &str = "╔";

// const right_up: &str = "╚";
// const left_up: &str = "╝";


// const line_up_down: &str = "║";
// const line: &str = "══";

fn cmd_goto(x: i16, y: i16) {
    CURSOR.goto(x as u16, y as u16).unwrap();
}

fn printc(txt: &str, fore_color: crossterm::Color, back_color: crossterm::Color) {
    let style = style(txt).with(fore_color).on(back_color);
    print!("{}", style);
    flush_stdout();
}   


static mut TEMPO: bool = false;
fn draw_game(snake: &Snake, points: i32, apple: Point){    
    let gap = "  ";
    
    cmd_goto(0, 0);
    printc(&" ".repeat(PLAYABLE_SIZE_MAX as usize * 2), crossterm::Color::Black, crossterm::Color::DarkGreen);
    
    
    let mut score_menu = String::from("Points: ") + &points.to_string();
    score_menu += gap;
    score_menu += &("Game Speed: ".to_owned() + &(21 - unsafe{ SPEED_MULT }).to_string());
    score_menu += gap;
    score_menu += &("Game Version: ".to_owned() + &GAME_VERSION.to_string());
    
    if unsafe{ GAME_PAUSED } {
        score_menu += " (PAUSED)";
    }    
    
    
    CURSOR.goto(0, 0).unwrap();
    print!("{}", style(score_menu).with(crossterm::Color::Black).on(crossterm::Color::DarkGreen));
    
    cmd_goto(apple.x * 2, apple.y - 1);
    printc(" /", Color::Green, Color::Black);
    cmd_goto(apple.x * 2, apple.y);
    printc("()", Color::Yellow, Color::Red);
    
    //let mut i = 0;
    for tail_part in snake.tail.iter() {        
        
        cmd_goto(tail_part.x * 2, tail_part.y);
        printc("88", Color::White, Color::DarkGrey);
        
        //i += 1;
    }
    
    cmd_goto(snake.head_pos.x * 2, snake.head_pos.y);
    let head: &str;    
    if unsafe{
        TEMPO = !TEMPO;
        TEMPO
    } {
        match snake.direction {
            Direction::Up => head = "\\/",
            Direction::Down => head = "/\\",
            Direction::Left => head = ">8",
            Direction::Right => head = "8<",
            Direction::None => head = "  ",
        }
    }else {
        match snake.direction {
            Direction::Up => head = "||",
            Direction::Down => head = "||",
            Direction::Left => head = "=8",
            Direction::Right => head = "8=",
            Direction::None => head = "  ",
        }
    }
    
    
    printc(head, Color::White, Color::Black);
}

fn sleep(ms: u64) {
    thread::sleep(time::Duration::from_millis(ms));
}

fn flush_stdout() {
    io::stdout().flush().unwrap();
}

#[derive(Copy, Clone)]
struct Point {
    x: i16,
    y: i16,
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}


#[derive(Clone)]
struct Snake { 
    direction: Direction, 
    head_pos: Point, 
    tail: Vec<Point> 
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None
}



#[macro_export]
macro_rules! input {
    () => {
        {
            let mut buf = String::new();
            STDIN.lock().read_line(&mut buf).unwrap();
            
            buf[..buf.len() - 2].to_string()
        }
    };
    ($x:expr) => {
        {
            print!($x);
            flush_stdout!();
            input!()
        }
    };
    
    ($x:expr, $($y:expr),+) => {
        {
            print!($x, $($y),+);
            flush_stdout!();
            input!()
        }
    }
}










      