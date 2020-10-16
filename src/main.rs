extern crate clap;
extern crate os_pipe;

use std::env;
use std::fs;
use std::path;
use std::process::Command;

use clap::Clap;

/// Generate dynamic, scripting language projects with dependencies for
/// quick CLI feedback loops.
#[derive(Clap)]
#[clap(version = "1.0", author = "Alex M. <alex41290@gmail.com>")]
struct Opts {
    #[clap(default_value = ".")]
    /// Path to create project
    path: String,
    #[clap(short)]
    /// Drop into REPL after building
    shell: bool,
}

const MIX_EXS: &str = r#"
defmodule Script.Mixfile do
  use Mix.Project

	def application do
		[applications: [:httpoison], extra_applications: [:nimble_csv, :jason]]
	end

	def project do
		[app: :script, version: "1.0.0", deps: deps()]
	end

	defp deps do
		[{:httpoison, "~> 1.7"}, {:jason, "~> 1.2"}, {:nimble_csv, "~> 1.1"}, {:floki, "~> 0.29"}]
	end
end
"#;

const PARSER: &str = r#"
NimbleCSV.define(CSVParser, separator: ",", escape: "\"")
NimbleCSV.define(TSVParser, separator: "\t", escape: "\"")

defmodule Parser do
  def parse_csv(file) do
		[ headers | data ] =
		file
		|> File.read!()
		|> CSVParser.parse_string(skip_headers: false)

		data
		|> Enum.map(&Enum.zip(headers, &1))
		|> Enum.map(&Map.new/1)
	end

	def to_csv(rows) when is_list(rows) do
		header =
		rows
		|> Enum.at(0)
		|> Map.keys()

		[header | Enum.map(rows, &Map.values/1)]
	end

	def write_csv(rows, file) when is_list(rows) do
		File.write!(file, CSVParser.dump_to_iodata(rows))
	end
end
"#;

fn main() {
    let opts = Opts::parse();

    println!("Generating mix project...");

    let folder_path = path::PathBuf::from(opts.path);

    let result: Result<(), String> = fs::create_dir_all(folder_path.clone())
        .map_err(|err| format!("Failed to create dir: {}", err))
        .and_then(|()| {
            env::set_current_dir(folder_path.clone())
                .map_err(|err| format!("Failed to set cwd: {}", err))
        })
        .and_then(|()| {
            fs::write("mix.exs", MIX_EXS)
                .map_err(|err| format!("Failed to write `mix.exs`: {}", err))
        })
        .and_then(|()| {
            fs::create_dir(path::PathBuf::from("lib"))
                .map_err(|err| format!("Failed to create `lib` dir: {}", err))
        })
        .and_then(|()| {
            fs::write(path::PathBuf::from("lib").join("parser.ex"), PARSER)
                .map_err(|err| format!("Failed to write `lib/parser.ex`: {}", err))
        });

    if let Err(error) = result {
        panic!("ahhhhhh: {}", error);
    }

    let stdout = os_pipe::dup_stdout().unwrap();
    let mut mix = match Command::new("mix")
        .args(&["do", "deps.get,", "deps.compile"])
        .stdout(stdout)
        .spawn()
    {
        Err(error) => panic!("Failed to spawn `mix` process: {}", error),
        Ok(process) => process,
    };

    let _ = mix.wait();

    if opts.shell {
        let stdout = os_pipe::dup_stdout().unwrap();
        let mut iex_command = match Command::new("iex")
            .args(&["-S", "mix"])
            .stdout(stdout)
            .stdin(os_pipe::dup_stdin().unwrap())
            .spawn()
        {
            Err(error) => panic!("Failed to run `iex` shell: {}", error),
            Ok(process) => process,
        };

        let _ = iex_command.wait();
    }
}
