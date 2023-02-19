mod response;
use response::{Args, ChatResponse, Parser};

use reqwest::blocking::Client;

use dirs::home_dir;
use rand::{self, Rng};

use termion::{
    self,
    clear,
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
};

use std::{
    fs,
    io::{self, Write},
    process::Command,
    time::Duration,
};

use stoppable_thread::{self, StoppableHandle};

const API_URL: &str = "https://api.openai.com/v1/completions";

const MESSAGE_SIZE: usize = 10;
const LOADING_MESSAGE: [&str; MESSAGE_SIZE] = [
    "Bleep bloop, the machine is learning",
    "You better not be cheating on homework",
    "Waiting on openai",
    "Hopefully we don't timeout",
    "Requesting a response",
    "This is not the droid you're looking for",
    "I'm not a bot, I swear",
    "Accessing the culmination of mankind's acheivments",
    "Hey Google, show me this guy's balls",
    "Hacking the mainframe",
];

fn get_api_key() -> Result<String, io::Error> {
    let mut stdout = io::stdout();
    let _ = stdout.flush();

    let config_folder = home_dir().unwrap().join(".config/rgpt");
    let config_file = config_folder.join("secret");

    let api_key = if config_file.exists() {
        fs::read_to_string(config_file)?
    } else {
        let mut buf = String::new();

        print!("Enter your api key: ");

        let _ = stdout.flush();
        io::stdin().read_line(&mut buf)?;
        let _ = stdout.flush();

        fs::create_dir_all(config_folder)?;
        fs::write(config_file, &buf)?;

        buf
    };

    Ok(api_key.trim().to_string())
}

fn askgpt(
    payload: &serde_json::Value,
    auth: &String,
) -> Result<ChatResponse, Box<dyn std::error::Error>> {
    let req = Client::new()
        .post(API_URL)
        .bearer_auth(auth)
        .json(&payload)
        .send()?;

    if req.status() != 200 {
        return Err(format!("Request failed with status: {}", req.status()).into());
    }

    let mut response = req.json::<ChatResponse>()?;
    response.choices[0].text = response.choices[0].text.trim().to_string();

    Ok(response)
}

fn animate(s: &str, speed: Option<u64>) -> Result<(), io::Error> {
    print!("{}\r", clear::CurrentLine);

    let mut stdout = std::io::stdout();
    let sleep = speed.unwrap_or(25);

    for c in s.chars() {
        print!("{}", c);
        let _ = stdout.flush();
        std::thread::sleep(Duration::from_millis(sleep));
    }
    println!();

    Ok(())
}

fn loading_message() -> StoppableHandle<()> {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..MESSAGE_SIZE);

    stoppable_thread::spawn(move |stopped| {
        let mut stdout = std::io::stdout();
        while !stopped.get() {
            let _ = stdout.flush();
            print!("{}{} {}", clear::CurrentLine, std::char::from_u32(0x1F916).unwrap(), LOADING_MESSAGE[index]);
            let _ = stdout.flush();

            for c in "...".chars() {
                let _ = stdout.flush();
                std::thread::sleep(Duration::from_millis(500));
                print!("{}", c);
                let _ = stdout.flush();
            }

            print!("\r");
        }
    })
}

fn confirm_exec() -> Result<bool, io::Error> {
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut stdin = termion::async_stdin().keys();
    
    stdout.flush()?;
    write!(stdout, "Execute command? [Y/N]:  ")?;
    stdout.flush()?;

    loop {
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            write!(stdout, "{}{}", termion::cursor::Left(1), clear::AfterCursor)?;
            stdout.flush()?;
            let res = match key {
                Key::Char('y') | Key::Char('Y') => true,
                Key::Char('n') | Key::Char('N') => false,
                Key::Char(k) => {
                    write!(stdout, "{}", k)?;
                    stdout.flush()?;
                    continue;
                },
                _ => { continue; }
            };

            write!(stdout, "{}", clear::CurrentLine)?;
            stdout.flush()?;
            return Ok(res);
        }

        std::thread::sleep(Duration::from_millis(25));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (payload, execute) = Args::parse().serialize();
    let api_key = get_api_key()?;

    let handle = loading_message();
    let response = askgpt(&payload, &api_key)?;
    handle.stop().join().unwrap();

    std::thread::sleep(Duration::from_millis(500));

    animate(&response.choices[0].text, None)?;

    if execute && confirm_exec()? {
        let mut output = Command::new("bash")
            .arg("-c")
            .arg(&response.choices[0].text)
            .spawn()?;

        output.wait()?;
    }

    Ok(())
}
