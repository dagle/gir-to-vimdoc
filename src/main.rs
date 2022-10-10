use std::str::FromStr;
use std::fs;

mod library;
mod parse;
mod lang;

use clap::Parser;

#[derive(Clone)]
enum Lang {
    Python,
    Lua,
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lua" => Ok(Lang::Lua),
            "python" => Ok(Lang::Python),
            lang => {
                let ret = format!("{} not supported", lang);
                Err(ret)
            }
        }
    }
}


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(required = true)]
    #[clap(short, long, value_name = "lua|python")]
    lang: Lang,

    #[clap(long, value_name = "Code|CodeDoc|Full")]
    #[clap(default_value_t = lang::Level::Code)]
    level: lang::Level,

    // generates all files found in "/usr/share/gir-1.0/" but overrides if file is 
    // found locally.
    #[clap(long)]
    #[clap(default_value_t = false)]
    gen_all: bool,

    // #[clap(required = true)]
    filename: Option<String>,
}

fn main() -> std::io::Result<()>{
    let args = Cli::parse();
    match args.lang {
        Lang::Python => {
            Ok(())
        },
        Lang::Lua => {
            let cg = lang::lua::LuaCodegen::new(lang::Level::Code);
            if args.gen_all {
                let paths = fs::read_dir("/usr/share/gir-1.0/")?;
                for path in paths {
                    let osstr = path.expect("Couldn't read filename").file_name();
                    let filename = osstr.to_str().expect("Couldn't read filename");
                    cg.gen(filename)?;
                }
            } else {
                let filename = args.filename.expect("Missing filename");
                cg.gen(&filename)?;
            }
            Ok(())
        },
    }
}
