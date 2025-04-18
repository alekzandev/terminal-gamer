use rusty_audio::Audio;
use terminal_gamer::{frame::{self, Drawable}, render, player::Player};
use std::{
    {io,thread},
    time::Duration,
    sync::mpsc,
    error::Error,
};
use crossterm::{
    event::{self, Event, KeyCode},
    ExecutableCommand,
    cursor::{Hide, Show},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "assets/sounds/explode.wav");
    audio.add("lose", "assets/sounds/lose.wav");
    audio.add("move", "assets/sounds/move.wav");
    audio.add("pew", "assets/sounds/pew.wav");
    audio.add("startup", "assets/sounds/startup.wav");
    audio.add("win", "assets/sounds/win.wav");

    // Play audio
    audio.play("startup");


     // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let current_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &current_frame, false);
            last_frame = current_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    'gameloop: loop {
        // per-frame unit
        let mut current_frame = frame::new_frame();

        // Input
        while event::poll(Duration::default())? {

            if let Event::Key(key_event) = event::read()?{
                match key_event.code {
                    KeyCode::Left => {
                        player.move_left();
                        // audio.play("move");
                    },
                    KeyCode::Right => {
                        player.move_right();
                        // audio.play("move");
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    _ => {}
                }
            }
        }
        // Draw & render
        player.draw(&mut current_frame);
        let _ = render_tx.send(current_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?; 
    Ok(())
}
