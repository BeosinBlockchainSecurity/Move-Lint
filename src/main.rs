use clap::Parser;

fn main() {
    let args = move_lint::Config::parse();
    let ast = match move_lint::gen_move_ast(args.package_path, args.build_config) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{:#?}", err);
            std::process::exit(1);
        }
    };
    let issues = match move_lint::move_lint(args.lint_config, &ast) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{:#?}", err);
            std::process::exit(1);
        }
    };
    if args.json {
        match serde_json::to_string(&issues.to_vec()) {
            Ok(s) => println!("{s}"),
            Err(err) => {
                eprintln!("{:#?}", err);
                std::process::exit(1);
            }
        };
    } else {
        println!("{:#?}", issues);
    }
}
