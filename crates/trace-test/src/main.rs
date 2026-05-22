mod gen;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 || args[1] != "gen" {
        eprintln!("Usage: trace-test gen <count> <output-path>");
        eprintln!("Example: trace-test gen 10000 ./test-vault-10k");
        std::process::exit(1);
    }

    let count: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("error: <count> must be a positive integer");
        std::process::exit(1);
    });

    let path = std::path::PathBuf::from(&args[3]);

    eprintln!("Generating {count} notes in {} ...", path.display());
    gen::generate_vault(count, &path).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    eprintln!("Done.");
}
