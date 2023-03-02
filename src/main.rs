mod response;
use response::{Args, ChatResponse, Parser};

use dirs::home_dir;
use futures_util::{select, FutureExt, StreamExt};
use reqwest::Client;
use tokio::io::AsyncWriteExt;

use crossterm::{
    cursor,
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use std::{
    fs::{self, File},
    io::{self, stdout, BufRead, BufReader, Write},
    process::Command,
    time::Duration,
};

type GptError = Box<dyn std::error::Error>;

const API_URL: &str = "https://api.openai.com/v1/completions";
const API_KEY_LEN: usize = 51;

fn get_api_key() -> Result<String, GptError> {
    let config_folder = home_dir().unwrap_or("rgpt".into()).join(".config/rgpt");
    let config_file = config_folder.join("secret");

    let key = if config_file.exists() {
        let mut reader = BufReader::with_capacity(API_KEY_LEN, File::open(config_file)?);
        let buf = reader.fill_buf()?;

        std::str::from_utf8(buf)?.into()
    } else {
        fs::create_dir_all(config_folder)?;

        let key =
            rpassword::prompt_password(format!("{}Enter your API key: ", cursor::SavePosition))?;
        execute!(
            stdout(),
            cursor::RestorePosition,
            Clear(ClearType::UntilNewLine)
        )?;

        fs::write(config_file, &key)?;

        key
    };

    Ok(key)
}

async fn confirm_exec() -> Result<bool, io::Error> {
    let mut stdin = EventStream::new();
    let mut stdout = io::stdout();

    print!("{}Execute command? [Y/N]  ", cursor::SavePosition);
    stdout.flush()?;

    loop {
        let mut event = stdin.next().fuse();
        let res = select! {
            maybe_event = event => {
                std::thread::sleep(Duration::from_millis(100));
                execute!(stdout, cursor::MoveLeft(1), Clear(ClearType::UntilNewLine))?;
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => std::process::exit(0),
                            (KeyCode::Char('y'), _) | (KeyCode::Char('Y'), _) => true,
                            (KeyCode::Char('n'), _) | (KeyCode::Char('N'), _) => false,
                            (KeyCode::Char(c), _) => {
                                print!("{c}");
                                stdout.flush()?;
                                continue;
                            }
                            _ => continue,
                        }
                    },
                    Some(Ok(_)) => continue,
                    Some(Err(_)) => continue,
                    _ => false,
                }
            }
        };

        execute!(
            stdout,
            cursor::RestorePosition,
            Clear(ClearType::UntilNewLine)
        )?;
        stdout.flush()?;

        return Ok(res);
    }
}

async fn askgpt(payload: &serde_json::Value, auth: String) -> Result<String, GptError> {
    let response = Client::new()
        .post(API_URL)
        .bearer_auth(auth)
        .json(&payload)
        .send()
        .await?;

    if response.status() != 200 {
        return Err(format!(
            "Request to OpenAi failed with status: {}",
            response.status()
        )
        .into());
    }

    let mut body = response.bytes_stream().skip(2);
    let mut chat = String::with_capacity(1024);
    let mut stdout = tokio::io::stdout();

    while let Some(chunk) = body.next().await {
        let body = chunk?;
        let body = body
            .strip_prefix(b"data: ")
            .unwrap_or_else(|| body.as_ref());

        match serde_json::from_slice::<ChatResponse>(body) {
            Ok(r) => {
                let text = &r.choices[0].text;
                chat.push_str(text);
                stdout.write_all(text.as_bytes()).await?;
                stdout.flush().await?;
            }
            Err(_) => continue, // TODO (Carter) only certain errors should be handled here
        };
    }

    println!();

    Ok(chat)
}

#[tokio::main]
async fn main() -> Result<(), GptError> {
    let (payload, execute) = Args::parse().serialize();
    let api_key = get_api_key()?;

    let chat = askgpt(&payload, api_key).await?;

    enable_raw_mode()?;
    if execute && confirm_exec().await? {
        Command::new("bash").arg("-c").arg(&chat).spawn()?.wait()?;
    }
    disable_raw_mode()?;

    Ok(())
}
