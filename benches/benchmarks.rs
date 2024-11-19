#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use puz::parse_puz;
    use std::fs;
    use test::{black_box, Bencher};

    #[bench]
    fn bench_example_files(b: &mut Bencher) {
        b.iter(|| {
            for i in 1..100 {
                let test_files = fs::read_dir("testfiles").expect("no files in dir");
                for file in test_files {
                    black_box(parse_puz(file.unwrap().path().to_str().unwrap()));
                }
            }
        });
    }
}
