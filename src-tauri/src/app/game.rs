use tauri::{AppHandle, command, Manager, State};
use std::process::{Command as ProcessCommand, Stdio};
use std::io::{BufReader, BufRead, Read};
use std::thread;
use crate::app::config;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use portable_pty::{CommandBuilder, native_pty_system, PtySize};
use crate::app::app_state::AppState;

#[command]
pub fn start_game(app_handle: AppHandle, app_state: State<'_, AppState>) {
    let stop_signal = app_state.stop_game.clone();

    thread::spawn(move || {
        let config = config::get_config();
        let mut game_path = config.game_path;
        game_path.push_str("/StardewModdingAPI.exe");

        let pty_system = native_pty_system();

        let mut pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 2000,
            pixel_width: 0,
            pixel_height: 0,
        }).unwrap();

        let cmd = CommandBuilder::new(game_path);
        let mut child = pair.slave.spawn_command(cmd).unwrap();

        let reader = pair.master.try_clone_reader().unwrap();
        let mut reader = BufReader::new(reader);
        let mut lines = reader.lines();

        // Regex to match shell title escape sequences
        let title_regex = Regex::new(r"\x1b\]0;.*?\x07").unwrap();

        println!("Hier");  // Optional: for debugging

        loop {
            if let Ok(mut lock) = stop_signal.lock() {
                if !*lock {
                    child.kill().unwrap();
                    drop(child);
                    drop(pair.slave);
                    drop(pair.master);
                    *lock = true;
                    break;
                }
            }

            if let Some(Ok(line)) = lines.next() {
                println!("{}", &line);  // Optional: for debugging
                let clean_line = title_regex.replace_all(&line, "");
                if !clean_line.is_empty() {
                    let html_line = ansi_to_html::convert(&clean_line.to_string()).unwrap();
                    app_handle.emit_all("console", html_line).expect("Failed to emit event");
                }
            }
        }
    });
}

#[command]
pub fn stop_game(app_state: State<'_, AppState>) {
    let mut signal = app_state.stop_game.lock().unwrap();
    *signal = false;
}

pub fn ssd(app_handle: AppHandle) {
    let config = config::get_config();
    let game_path = config.game_path;
    let game_executable = format!("{}/StardewModdingAPI.exe", game_path);
    println!("{}", game_executable);

    let mut child = ProcessCommand::new(game_executable)
        .arg("--color=always")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdout = child.stdout.take().expect("Failed to capture stdout");

    // Move reading and event emission to a separate thread
    let app_handle_clone = app_handle.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let html_line = ansi_to_html::convert(&line).unwrap();
                    println!("{}", html_line);
                    app_handle_clone.emit_all("console", html_line).expect("Failed to emit event");
                },
                Err(e) => {
                    eprintln!("Error reading line from process output: {}", e);
                    break;
                }
            }
        }
    });

    /*
    while let Some(Ok(line)) = lines.next() {
        let clean_line = title_regex.replace_all(&line, "");

        if !clean_line.is_empty() {
            let html_line = ansi_to_html::convert(&clean_line.to_string()).unwrap();
            println!("{}", html_line);  // Optional: for debugging
            app_handle.emit_all("console", html_line).expect("Failed to emit event");
        }
    }*/
}