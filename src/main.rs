mod response;
use response::{Args, ChatResponse, Parser};

use dirs::home_dir;
use futures_util::StreamExt;
use reqwest::Client;

use termion::{self, clear, cursor, event::Key, input::TermRead, raw::IntoRawMode};

use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::Command,
    time::Duration,
};

use tokio::io::AsyncWriteExt;

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

        let key = rpassword::prompt_password(format!("{}Enter your API key: ", cursor::Save))?;
        print!("{}{}", cursor::Restore, clear::AfterCursor);
        io::stdout().flush()?;

        fs::write(config_file, &key)?;

        key
    };

    Ok(key)
}

fn confirm_exec() -> Result<bool, io::Error> {
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut stdin = termion::async_stdin().keys();

    stdout.flush()?;
    print!("{}Execute command? [Y/N]:  ", cursor::Save);
    stdout.flush()?;

    loop {
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            print!("{}{}", termion::cursor::Left(1), clear::AfterCursor);
            stdout.flush()?;

            let res = match key {
                Key::Char('y') | Key::Char('Y') => true,
                Key::Char('n') | Key::Char('N') => false,
                Key::Char(k) => {
                    print!("{k}");
                    stdout.flush()?;
                    continue;
                }
                _ => {
                    continue;
                }
            };

            print!("{}{}", cursor::Restore, clear::AfterCursor);
            stdout.flush()?;

            return Ok(res);
        }

        std::thread::sleep(Duration::from_millis(25));
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

                let text = text.trim_start_matches('\n');

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (payload, execute) = Args::parse().serialize();
    let api_key = get_api_key()?;

    let chat = askgpt(&payload, api_key).await?;

    if execute && confirm_exec()? {
        Command::new("bash").arg("-c").arg(&chat).spawn()?.wait()?;
    }

    Ok(())
}
