use std::{io::Write, path::Path};

use rand::Rng;

const WORDS: &[&str] = &[
    "atlas", "beacon", "cipher", "delta", "epoch", "fusion", "ghost", "harbor", "index", "journal",
    "kernel", "latent", "matrix", "nexus", "orbit", "prism", "quorum", "relay", "signal", "trace",
    "unity", "vector", "warden", "xenon", "yield", "zenith", "anchor", "bridge", "canvas",
    "dagger", "ember", "flare", "glyph", "hinge", "infer", "juncture", "lantern", "mosaic",
    "notion", "opal",
];

pub fn generate_vault(count: usize, path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;

    let titles: Vec<String> = (0..count).map(|i| format!("note-{i:0>6}")).collect();
    let mut rng = rand::thread_rng();

    for (i, title) in titles.iter().enumerate() {
        let file_path = path.join(format!("{title}.md"));
        let mut f = std::fs::File::create(&file_path)?;

        writeln!(f, "# {title}")?;
        writeln!(f)?;

        let sentence_count = rng.gen_range(2..=5usize);
        for _ in 0..sentence_count {
            let words: Vec<&str> = (0..rng.gen_range(6..=14usize))
                .map(|_| WORDS[rng.gen_range(0..WORDS.len())])
                .collect();
            writeln!(f, "{}.", words.join(" "))?;
        }

        if count > 1 && rng.gen_bool(0.2) {
            writeln!(f)?;
            let link_count = rng.gen_range(1..=3usize).min(count - 1);
            for _ in 0..link_count {
                let mut target = rng.gen_range(0..count);
                if target == i {
                    target = (target + 1) % count;
                }
                writeln!(f, "See also [[{}]].", titles[target])?;
            }
        }

        if i > 0 && i % 10_000 == 0 {
            eprintln!("  {i}/{count}");
        }
    }

    Ok(())
}
