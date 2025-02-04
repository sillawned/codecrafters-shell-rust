use std::env;
use crate::utils::search_cmd;

pub const BUILTINS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];
// const CONTROL_OPERATORS: [&str; 12] = [
//     "\n", "&&", "||", "&", ";", ";;", ";&", ";;&", "|", "|&", "(", ")",
// ];
// const META_CHARACTERS: [&str; 10] = [" ", "\t", "\n", "|", "&", ";", "(", ")", "<", ">"];

pub fn execute_builtin(name: &str, args: &[String]) -> Result<(), String> {
    match name {
        "exit" => std::process::exit(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0)),
        "echo" => {
            println!("{}", args.join(" "));
            Ok(())
        }
        "pwd" => {
            println!("{}", env::current_dir().unwrap().display());
            Ok(())
        }
        "cd" => {
            let path = args.get(0).map_or(env::var("HOME").unwrap(), |s| s.clone());
            env::set_current_dir(&path).map_err(|e| e.to_string())
        }
        "type" => {
          if BUILTINS.contains(&args[0].as_str()) {
              println!("{} is a shell builtin", args[0]);
          } else if let Some(cmd_path) = search_cmd(&args[0], &std::env::var("PATH").unwrap()) {
              println!("{} is {}", args[0], cmd_path);
          } else {
              println!("{}: not found", args[0]);
          }
            Ok(())
        }
        _ => Err(format!("Unknown builtin: {}", name)),
    }
}