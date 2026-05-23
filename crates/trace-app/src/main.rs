// No `windows_subsystem = "windows"` here: keeping both debug and release as
// console-subsystem binaries gives identical CLI behavior. GUI mode calls
// FreeConsole() to detach the inherited console so no window lingers.

mod gen;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let headless = args.iter().any(|a| a == "-h" || a == "--headless");
    let do_gen = args.iter().any(|a| a == "-g");

    if do_gen {
        let count = args
            .windows(2)
            .find(|w| w[0] == "-g")
            .and_then(|w| w[1].parse::<usize>().ok())
            .unwrap_or(100);
        // Explicit -o is CWD-relative (normal CLI convention).
        // Default with no -o is exe-relative so the vault lands next to the binary.
        let output = args
            .windows(2)
            .find(|w| w[0] == "-o" || w[0] == "--output")
            .map(|w| std::env::current_dir().unwrap_or_default().join(&w[1]))
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("trace-gen")))
                    .unwrap_or_else(|| std::path::PathBuf::from("trace-gen"))
            });

        eprintln!("Generating {count} notes in {} ...", output.display());
        gen::generate_vault(count, &output).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            std::process::exit(1);
        });
        eprintln!("Done.");

        if headless {
            std::process::exit(0);
        }
        // Without -h: fall through and open the GUI.
    } else if headless {
        eprintln!("Usage:");
        eprintln!("  trace -g [count] [-o dir] [-h headless] Generate a synthetic vault");
        std::process::exit(0);
    }

    // GUI mode: detach from the console so no window lingers after launch.
    #[cfg(target_os = "windows")]
    unsafe {
        extern "system" {
            fn FreeConsole() -> i32;
        }
        FreeConsole();
    }

    trace_app_lib::run()
}
