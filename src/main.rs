use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufRead, stdout};
use std::path::Path;
use crossterm::{
    execute,
    terminal,
    cursor::{MoveUp, MoveDown, SavePosition, RestorePosition},
    style::Color, style::SetForegroundColor,style::SetBackgroundColor,
    event::{read, Event, KeyCode, KeyModifiers},
};

/// CLI options -i -o -n -r -p -v
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'i', long = "input")]
    input_file: String,
    #[arg(short = 'o', long = "output")]
    output_file: String,
    #[arg(short = 'n', long = "output_number", default_value_t = String::from(""))]
    output_number_file: String,
    #[arg(short = 'r', long = "reverse")]
    reverse: bool,
    #[arg(short = 'v', long = "view", default_value_t = String::from("all"))]
    view: String,
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
    if path=="-"{
        println!("{}",value.trim_end());
    }else{
        fs::write(path, value.trim_end())?;
    }
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

fn draw_menu(items: &[String], selected: usize, page_start: usize, page_size: usize, view:String) -> u16 {
    let mut upcnt = 0;

    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();

    if view=="all" {
        println!("\rNavigate: ↑ ↓ pageUp pageDown; Choose: space →> \n");
        upcnt += 1;
    }

    let end = usize::min(page_start + page_size, items.len());
    let mut downcnt:u16=0;
    let mut print_cnt=0;
    for (i, item) in items[page_start..end].iter().enumerate() {
        let idx = i + page_start;
        print_cnt+=1;
        if idx == selected {

            execute!(stdout(),SetBackgroundColor(Color::DarkBlue)).unwrap();
            print!("\r>>> {}", item);
            execute!(stdout(),SetBackgroundColor(Color::Reset)).unwrap();
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
        if view=="all" {
            println!(
                "\n\rpage {}/{}",
                page_start / page_size + 1,
                (items.len() + page_size - 1) / page_size
            );
            upcnt+=2;
        }
   }
    downcnt+=2;
    print!("\r");
    execute!(stdout(), MoveUp(1+upcnt)).unwrap();

    upcnt
}

fn save_cursor() {
    execute!(stdout(), SavePosition).unwrap();
}

fn restore_cursor() {
    execute!(stdout(), RestorePosition).unwrap();
}

fn main() -> io::Result<()> {

    let args = Args::parse();

    let mut items = load_lines(&args.input_file)?;
    if args.reverse {
      items.reverse();
    }


    if items.is_empty() {
        println!("Input file is empty");
        return Ok(());
    }


    let page_size = args.page_size;
    let view = args.view;
    let mut downcnt;
    let mut selected = 0;
    let mut page_start = 0;

    if let Some(idx) = restore_selected(&args.output_file, &items) {
        selected = idx;
        page_start = (selected / page_size) * page_size;
    }

    terminal::enable_raw_mode()?;
    save_cursor();
    // ---- Main user iteraction loop ----
    loop {
        restore_cursor();
        downcnt = draw_menu(&items, selected, page_start, page_size, view.clone());
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
                (KeyCode::PageUp, _) => {
                    let new_selected:i32 = (selected as i32) - (page_size as i32);
                    if new_selected >0{
                        selected   -= page_size;
                        page_start -= page_size
                    } else {
                        selected = 0;
                        page_start=0;
                    }
                }
                (KeyCode::PageDown, _) => {
                    let new_selected:i32 = (selected as i32)+ (page_size as i32);
                    let max_:i32 = (items.len()as i32)-1;
                    if new_selected < max_ && max_>0
                    {
                        selected   += page_size;
                        page_start += page_size
                    } else {
                        selected =  max_ as usize;
                        let npages = (selected/page_size) as usize;
                        page_start = page_size * npages;
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
                    if let Err(e) = save_selected(&args.output_file, &items[selected]) {
                        println!("Could not save selected string into the file: {}", e);
                    }

                    if args.output_number_file!="" {
                        let human_idx = selected + 1;
                        if let Err(e) = save_selected(&args.output_number_file, &human_idx.to_string()) {
                            println!("Could not save index into the file: {}", e);
                        }
                    }
                    break;
                }

                // Ctrl-C
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    println!("\nCtrl-C");
                    // if let Err(e) = save_selected(save_file, "Ctrl-C") {
                    //     println!("Could not save string into the file: {}", e);
                    // }
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
