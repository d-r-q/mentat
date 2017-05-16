// Copyright 2017 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use combine::{
    eof, 
    many1, 
    satisfy, 
    sep_end_by, 
    token, 
    Parser
};
use combine::char::{
    space, 
    spaces, 
    string
};
use combine::combinator::{
    choice, 
    try
};

use errors as cli;

pub static HELP_COMMAND: &'static str = &"help";
pub static OPEN_COMMAND: &'static str = &"open";
pub static CLOSE_COMMAND: &'static str = &"close";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Transact(Vec<String>),
    Query(Vec<String>),
    Help(Vec<String>),
    Open(String),
    Close,
}

impl Command {
    /// is_complete returns true if no more input is required for the command to be successfully executed.
    /// false is returned if the command is not considered valid. 
    /// Defaults to true for all commands except Query and Transact.
    /// TODO: for query and transact commands, they will be considered complete if a parsable EDN has been entered as an argument
    pub fn is_complete(&self) -> bool {
        match self {
            &Command::Query(_) |
            &Command::Transact(_) => false,
            &Command::Help(_) |
            &Command::Open(_) |
            &Command::Close => true
        }
    }
}

pub fn command(s: &str) -> Result<Command, cli::Error> {
    let arguments = || sep_end_by::<Vec<_>, _, _>(many1(satisfy(|c: char| !c.is_whitespace())), many1::<Vec<_>, _>(space())).expected("arguments");

    let help_parser = string(HELP_COMMAND)
                    .with(spaces())
                    .with(arguments())
                    .map(|args| {
                        Ok(Command::Help(args.clone()))
                    });

    let open_parser = string(OPEN_COMMAND)
                    .with(spaces())
                    .with(arguments())
                    .map(|args| {
                        if args.len() < 1 {
                            return Err(cli::ErrorKind::CommandParse("Missing required argument".to_string()).into());
                        }
                        if args.len() > 1 {
                            return Err(cli::ErrorKind::CommandParse(format!("Unrecognized argument {:?}", args[1])).into());
                        }
                        Ok(Command::Open(args[0].clone()))
                    });

    let close_parser = string(CLOSE_COMMAND)
                    .with(arguments())
                    .map(|args| {
                        if args.len() > 0 {
                            return Err(cli::ErrorKind::CommandParse(format!("Unrecognized argument {:?}", args[0])).into());
                        }
                        Ok(Command::Close)
                    });

    spaces()
    .skip(token('.'))
    .with(choice::<[&mut Parser<Input = _, Output = Result<Command, cli::Error>>; 3], _>
          ([&mut try(help_parser),
            &mut try(open_parser),
            &mut try(close_parser),]))
        .skip(spaces())
        .skip(eof())
        .parse(s)
        .map(|x| x.0)
        .unwrap_or(Err(cli::ErrorKind::CommandParse(format!("Invalid command {:?}", s)).into()))
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_help_parser_multiple_args() {
        let input = ".help command1 command2";
        let cmd = command(&input).expect("Expected help command");
        match cmd {
            Command::Help(args) => {
                assert_eq!(args, vec!["command1", "command2"]);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_help_parser_dot_arg() {
        let input = ".help .command1";
        let cmd = command(&input).expect("Expected help command");
        match cmd {
            Command::Help(args) => {
                assert_eq!(args, vec![".command1"]);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_help_parser_no_args() {
        let input = ".help";
        let cmd = command(&input).expect("Expected help command");
        match cmd {
            Command::Help(args) => {
                let empty: Vec<String> = vec![];
                assert_eq!(args, empty);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_help_parser_no_args_trailing_whitespace() {
        let input = ".help ";
        let cmd = command(&input).expect("Expected help command");
        match cmd {
            Command::Help(args) => {
                let empty: Vec<String> = vec![];
                assert_eq!(args, empty);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_open_parser_multiple_args() {
        let input = ".open database1 database2";
        let err = command(&input).expect_err("Expected an error");
        assert_eq!(err.to_string(), "Unrecognized argument \"database2\"");
    }

    #[test]
    fn test_open_parser_single_arg() {
        let input = ".open database1";
        let cmd = command(&input).expect("Expected open command");
        match cmd {
            Command::Open(arg) => {
                assert_eq!(arg, "database1".to_string());
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_open_parser_path_arg() {
        let input = ".open /path/to/my.db";
        let cmd = command(&input).expect("Expected open command");
        match cmd {
            Command::Open(arg) => {
                assert_eq!(arg, "/path/to/my.db".to_string());
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_open_parser_file_arg() {
        let input = ".open my.db";
        let cmd = command(&input).expect("Expected open command");
        match cmd {
            Command::Open(arg) => {
                assert_eq!(arg, "my.db".to_string());
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn test_open_parser_no_args() {
        let input = ".open";
        let err = command(&input).expect_err("Expected an error");
        assert_eq!(err.to_string(), "Missing required argument");
    }

    #[test]
    fn test_close_parser_with_args() {
        let input = ".close arg1";
        let err = command(&input).expect_err("Expected an error");
        assert_eq!(err.to_string(), format!("Invalid command {:?}", input));
    }

    #[test]
    fn test_close_parser_no_args() {
        let input = ".close";
        let cmd = command(&input).expect("Expected close command");
        match cmd {
            Command::Close => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn test_close_parser_no_args_trailing_whitespace() {
        let input = ".close ";
        let cmd = command(&input).expect("Expected close command");
        match cmd {
            Command::Close => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn test_parser_preceeding_trailing_whitespace() {
        let input = " .close ";
        let cmd = command(&input).expect("Expected close command");
        match cmd {
            Command::Close => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn test_command_parser_no_dot() {
        let input = "help command1 command2";
        let err = command(&input).expect_err("Expected an error");
        assert_eq!(err.to_string(), format!("Invalid command {:?}", input));
    }

    #[test]
    fn test_command_parser_invalid_cmd() {
        let input = ".foo command1";
        let err = command(&input).expect_err("Expected an error");
        assert_eq!(err.to_string(), format!("Invalid command {:?}", input));
    }
}