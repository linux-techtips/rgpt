pub use clap::Parser;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub finish_reason: String,
    pub index: i32,
    pub logprobs: Option<serde_json::Value>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Content>,
    pub created: i32,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: Usage
}

#[derive(Parser, Debug)]
#[command(name="rgpt")]
#[command(author="linux-techtips")]
#[command(version="1.0")]
#[command(about="A simple CLI for OpenAI's GPT-3 API", long_about = None)]
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
            }
        ), self.execute)
    }
}