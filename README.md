# RGPT
#### A very fast command-line client for OpenAi's various text models.

## Installation
```sh
$ cargo install rgpt
```
On your first use, rgpt will prompt you for your api key. You can generate one [here](https://platform.openai.com/account/api-keys)
  
## Documentation
### Chatting

```sh
$ rgpt "What is your name"

My name is Sam.

$ rgpt "What is the best programming language"

Rust.
```

###### *Disclaimer*: Responses to the prompts shown above are modified to correct for industry biases

### Shell Commands

`rgpt` can take a description of a command and turn it into shell code

```sh
$ rgpt -s "What is the command to install polars for python"
pip install polars

```

`rgpt` is also capable of executing commands in a bash capable shell environments

```sh
$ rgpt -se "What is the command to add serde to a rust project"
cargo add serde
Execute command? [Y/N]: 

```

### Arguments
```sh
A simple CLI for OpenAI's various text models

Usage: rgpt [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>

Options:
      --model <MODEL>            [default: text-davinci-003]
      --max-tokens <MAX_TOKENS>  [default: 1024]
  -s, --shell
  -e, --execute
  -h, --help                     Print help
  -V, --version                  Print version
```

## Future Plans

- [ ] More options for API
- [ ] Improved error handling
- [ ] User-defined prompts
- [ ] Sharing of prompts
- [ ] Repl environment (Indev)
- [ ] Context and chat storage

## License
Copyright (c) 2023 MIT License

## Contact
[Issues](https://github.com/linux-techtips/rgpt/issues)
