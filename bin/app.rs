use std::{
    env,
    fs::File,
    io::{self, BufRead, Read, Write, stdout},
    process::exit,
};
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Welcome to K-Leng version 0.0.1");
        loop {
            print!(">>> ");
            stdout().flush()?;

            let mut buffer = String::new();
            let mut stdin_lock = io::stdin().lock();
            stdin_lock.read_line(&mut buffer)?;
            let input = buffer.trim();
            handle_command(input);
        }
    }

    if args.len() == 2 {
        let _ = args.first();
        let filename = args.first().unwrap();
        match File::open(filename) {
            Ok(mut file) => {
                let mut buff = [0u8; 1000];
                file.read_exact(&mut buff)?;
                println!("{buff:?}");
            }
            _ => eprintln!("file {} do not exist", filename),
        }
    }

    Ok(())
}

fn handle_command(input: &str) {
    match input {
        ".help" => println!("type .exit for exit"),
        ".exit" => exit(0),
        _ => {
            unimplemented!()
        }
    }
}
