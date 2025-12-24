use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufRead, stdout};
use std::path::Path;
//use std::env;

use crossterm::{
    execute,
    terminal,
    cursor::{MoveUp, MoveDown, SavePosition, RestorePosition},
    style::Color, style::SetForegroundColor,style::SetBackgroundColor,
    event::{read, Event, KeyCode, KeyModifiers},
};

/// CLI options -i -o -r
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'i', long = "input")]
    input_file: String,
    #[arg(short = 'o', long = "output")]
    output_file: String,
    #[arg(short = 'r', long = "reverse")]
    reverse: bool,
    #[arg(short = 'p', long = "page_size", default_value_t = 10)]
    page_size: usize,
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

fn draw_menu(items: &[String], selected: usize, page_start: usize, page_size: usize) -> u16 {
    let mut upcnt = 0;

    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    
    println!("\r↑↓/←→ навигация; Space/→> выбор;\n");
    upcnt+=1;

    let end = usize::min(page_start + page_size, items.len());
    let mut downcnt:u16=0;
    let mut print_cnt=0;
    for (i, item) in items[page_start..end].iter().enumerate() {
        let idx = i + page_start;
        print_cnt+=1;
        if idx == selected {

            execute!(
                    stdout(),
                    SetBackgroundColor(Color::DarkBlue)
                ).unwrap();

            print!("\r>>> {}", item);

            execute!(
                    stdout(),
                    SetBackgroundColor(Color::Reset)
                ).unwrap();

            println!("");

	        downcnt = i as u16;

        } else {

            println!("\r    {}", item);

        }
	    upcnt+=1;

    }
    if print_cnt<page_size {
        let dif=page_size-print_cnt;
        for n in 1..=dif {
            println!("");
            upcnt+=1;
        }
    }



    if (items.len() + page_size - 1) / page_size >1 {
        println!(
	    "\n\rСтраница {}/{}",
    	    page_start / page_size + 1,
            (items.len() + page_size - 1) / page_size
	);
	upcnt+=2;
   }
    downcnt+=2;
    print!("\r"); 
    execute!(stdout(), MoveUp(1+upcnt)).unwrap();
    
    upcnt
}

fn save_cursor() {
    print!("\x1b[s");
}

fn restore_cursor() {
    print!("\x1b[u");
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


    let page_size = args.page_size;
    let save_file = &args.output_file;
    let mut downcnt;
    // ---- Selection ----
    let mut selected = 0;
    let mut page_start = 0;

    // Restore previous selection (if exists)
    if let Some(idx) = restore_selected(save_file, &items) {
        selected = idx;
        page_start = (selected / page_size) * page_size;
    }

    terminal::enable_raw_mode()?;
    save_cursor();
    // ---- Main loop ----
    loop {
        restore_cursor();
        downcnt = draw_menu(&items, selected, page_start, page_size);
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
                // (KeyCode::Char('q'), _) => {
                //     println!("\nВыход без выбора.");
                //     break;
                // }

                _ => {}
            },
            _ => {}
        }
    }
    execute!(stdout(), MoveDown(downcnt+1)).unwrap();
    terminal::disable_raw_mode()?;
    Ok(())
}
