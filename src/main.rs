use puz::parse_by_glob;

fn main() {
    let results = parse_by_glob("testfiles/nyt-puz/**/*.puz");
    println!("{}", results.expect("Failed to parse").len());
}