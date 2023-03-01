pub use clap::Parser;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Content {
    pub text: String,
    pub index: isize,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: isize,
    pub choices: Vec<Content>,
    pub model: String,
}

#[derive(Parser, Debug)]
#[command(name = "rgpt")]
#[command(author = "linux-techtips")]
#[command(version = "1.0")]
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
            self.prompt.to_owned()
        };
        (
            json!(
                {
                    "prompt": prompt,
                    "model": self.model,
                    "max_tokens": self.max_tokens,
                    "stream": true,
                }
            ),
            self.execute,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_serialize() {
        let json = json!({
            "prompt": "Hello world!",
            "model": "text-davinci-003",
            "max_tokens": 1024,
            "stream": true,
        });

        let (args, execute) = Args {
            prompt: "Hello world!".to_owned(),
            model: "text-davinci-003".to_owned(),
            max_tokens: 1024,
            shell: false,
            execute: false,
        }
        .serialize();

        assert_eq!(args, json);
        assert_eq!(execute, false);
    }

    #[test]
    fn test_response_deserialize() {
        let json = json!({
            "id": "1",
            "object": "response",
            "created": 1609459200,
            "choices": [
                {
                    "text": "Hello world!",
                    "index": 0,
                    "logprobs": null,
                    "finish_reason": null,
                },
            ],
            "model": "text-davinci-003",
        });

        let response = ChatResponse {
            id: "1".to_owned(),
            object: "response".to_owned(),
            created: 1609459200,
            choices: vec![Content {
                text: "Hello world!".to_owned(),
                index: 0,
                logprobs: None,
                finish_reason: None,
            }],
            model: "text-davinci-003".to_owned(),
        };

        assert_eq!(response, serde_json::from_value(json).unwrap());
    }
}
