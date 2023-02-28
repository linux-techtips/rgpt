pub use clap::Parser;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Content {
    pub text: String,
    pub index: isize,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: isize,
    pub completion_tokens: isize,
    pub total_tokens: isize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: isize,
    pub choices: Vec<Content>,
    pub model: String,
}

#[derive(Parser, Debug)]
#[command(name="rgpt")]
#[command(author="linux-techtips")]
#[command(version="1.0")]
#[command(about="A simple CLI for OpenAI's various text models", long_about = None)]
pub struct Args {
    pub prompt: String,
    #[clap(long, default_value = "text-davinci-003")]
    pub model: String,
    #[clap(long, default_value = "1024")]
    pub max_tokens: i32,
    #[clap(short, long)]
    pub shell: bool,
    #[clap(short, long)]
    pub execute: bool,
}

impl Args {
    pub fn serialize(&self) -> (serde_json::Value, bool) {
        let prompt = if self.shell {
            format!("Provide only shell command as output. {}.", self.prompt)
        } else {
            self.prompt.clone()
        };
        (json!(
            {
                "prompt": prompt,
                "model": self.model,
                "max_tokens": self.max_tokens,
                "stream": true,
            }
        ), self.execute)
    }
}