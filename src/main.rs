use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufRead, stdout};
use std::path::Path;
//use std::env;

use crossterm::{
    execute,
    terminal,
    event::{read, Event, KeyCode, KeyModifiers},
};


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input filename
    #[arg(short = 'i', long = "input")]
    input_file: String,

    /// Output filename
    #[arg(short = 'o', long = "output")]
    output_file: String,

    #[arg(short = 'r', long = "reverse")]
    reverse: bool,

}




fn load_lines(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let mut l = line?;
        l = l.trim_end().to_string(); // trim newline
        if !l.is_empty() {
            lines.push(l);
        }
    }

    Ok(lines)
}

fn save_selected(path: &str, value: &str) -> io::Result<()> {
    fs::write(path, value.trim_end())?;
    Ok(())
}

fn restore_selected(path: &str, items: &[String]) -> Option<usize> {
    if !Path::new(path).exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let value = content.trim_end();

    items.iter().position(|item| item == value)
}

fn draw_menu(items: &[String], selected: usize, page_start: usize, page_size: usize) {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();

    println!("\r↑↓/←→ навигация — Space/→ выбор — q выход — Ctrl-C выход\n");

    let end = usize::min(page_start + page_size, items.len());

    for (i, item) in items[page_start..end].iter().enumerate() {
        let idx = i + page_start;
        if idx == selected {
            println!("\r> {}", item);
        } else {
            println!("\r  {}", item);
        }
    }
    if (items.len() + page_size - 1) / page_size >1 {
    println!(
        "\n\rСтраница {}/{}",
        page_start / page_size + 1,
        (items.len() + page_size - 1) / page_size
    );
   }
}

fn main() -> io::Result<()> {
    // ---- Args ----
    let args = Args::parse();
    // ---- Load items ----
    let mut items = load_lines(&args.input_file)?;
    if args.reverse {
      items.reverse();
    }   
    if items.is_empty() {
        println!("Файл пустой.");
        return Ok(());
    }

    let page_size = 10;
    let save_file = &args.output_file;

    // ---- Selection ----
    let mut selected = 0;
    let mut page_start = 0;

    // Restore previous selection (if exists)
    if let Some(idx) = restore_selected(save_file, &items) {
        selected = idx;
        page_start = (selected / page_size) * page_size;
    }

    terminal::enable_raw_mode()?;

    // ---- Main loop ----
    loop {
        draw_menu(&items, selected, page_start, page_size);

        match read()? {
            Event::Key(event) => match (event.code, event.modifiers) {
                // Навигация
                (KeyCode::Up, _) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < page_start {
                            page_start = page_start.saturating_sub(page_size);
                        }
                    }
                }
                (KeyCode::Down, _) => {
                    if selected + 1 < items.len() {
                        selected += 1;
                        if selected >= page_start + page_size {
                            page_start += page_size;
                        }
                    }
                }
                (KeyCode::Left, _) => {
                    if selected > 0 {
                        selected -= 1;
                        if selected < page_start {
                            page_start = page_start.saturating_sub(page_size);
                        }
                    }
                }

                // Выбор: Space или →
                (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char(' '), _) => {
                    //println!("\n\rВы выбрали: {}\r", items[selected]);
                    if let Err(e) = save_selected(save_file, &items[selected]) {
                        println!("Не удалось сохранить выбор: {}", e);
                    }
                    break;
                }

                // Ctrl-C
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    println!("\nВыход по Ctrl-C.");
                    break;
                }

                // Выход через q
                (KeyCode::Char('q'), _) => {
                    println!("\nВыход без выбора.");
                    break;
                }

                _ => {}
            },
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
