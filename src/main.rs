use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};

use fnv::FnvHashMap;
use memmap::MmapOptions;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn main() -> Result<()> {
    let input = env::args().nth(1).expect("missing path to the input file");

    let input = File::open(input)?;

    let started = std::time::Instant::now();
    let output = Solver::new(input).solve(Variant::MmapParallel)?;
    let elapsed = started.elapsed();

    println!("{{{output}}}");
    eprintln!("{:.2}ms elapsed", elapsed.as_micros() as f64 / 1000.0);

    Ok(())
}

struct Measurement {
    min: f64,
    max: f64,
    sum: f64,
    count: f64,
}

impl Measurement {
    fn new(value: f64) -> Self {
        let (min, max, sum) = (value, value, value);
        let count = 1.0;
        Self {
            min,
            max,
            sum,
            count,
        }
    }

    fn update(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
        self.count += 1.0;
    }

    fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.count += other.count;
    }
}

#[allow(dead_code)]
enum Variant {
    Naive,
    MmapSingleThread,
    MmapParallel,
}

struct Solver {
    input: File,
}

impl Solver {
    fn new(input: File) -> Self {
        Self { input }
    }

    fn solve(&self, variant: Variant) -> Result<String> {
        let measurements = match variant {
            Variant::Naive => self.solve_naive(),
            Variant::MmapSingleThread => self.solve_mmap_single_thread(),
            Variant::MmapParallel => self.solve_mmap_parallel(),
        }?;

        let mut measurements = measurements.into_iter().collect::<Vec<_>>();

        measurements.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

        let output = measurements
            .into_iter()
            .map(|(name, measurement)| {
                format!(
                    "{name}={:.1}/{:.1}/{:.1}",
                    measurement.min,
                    measurement.sum / measurement.count,
                    measurement.max
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("{{{output}}}"))
    }

    fn solve_naive(&self) -> Result<FnvHashMap<String, Measurement>> {
        let mut measurements = FnvHashMap::<String, Measurement>::default();

        for line in BufReader::new(&self.input).lines() {
            let line = line.expect("broken line");

            let (name, value) = line.split_once(';').expect("incorrect line format");

            let value = value[0..value.len() - 1]
                .parse()
                .expect("error parsing value");

            if let Some(measurement) = measurements.get_mut(name) {
                measurement.update(value);
            } else {
                measurements.insert(name.to_owned(), Measurement::new(value));
            }
        }

        Ok(measurements)
    }

    fn solve_mmap_single_thread(&self) -> Result<FnvHashMap<String, Measurement>> {
        let mmap = unsafe { MmapOptions::new().map(&self.input)? };

        Ok(Self::segment_measurements(&mmap))
    }

    fn solve_mmap_parallel(&self) -> Result<FnvHashMap<String, Measurement>> {
        let mut measurements = FnvHashMap::<String, Measurement>::default();

        let mmap = unsafe { MmapOptions::new().map(&self.input)? };

        let collected = Self::find_segments(&mmap)
            .into_par_iter()
            .flat_map(|(from, to)| {
                Self::segment_measurements(&mmap[from..to])
                    .into_iter()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for (name, measurement) in collected {
            if let Some(existing) = measurements.get_mut(&name) {
                existing.merge(&measurement);
            } else {
                measurements.insert(name, measurement);
            }
        }

        Ok(measurements)
    }

    fn find_segments(mmap: &[u8]) -> Vec<(usize, usize)> {
        let mut bounds = Vec::with_capacity(rayon::current_num_threads());

        let mut prev = 0;
        for _ in 0..bounds.capacity() {
            let end = (prev + mmap.len() / bounds.capacity()).min(mmap.len() - 1);

            let end = end + memchr::memchr(b'\n', &mmap[end..]).expect("missing segment end") + 1;

            bounds.push((prev, end));

            prev = end;
        }

        bounds
    }

    fn segment_measurements(mmap: &[u8]) -> FnvHashMap<String, Measurement> {
        let mut measurements = FnvHashMap::<String, Measurement>::default();

        let mut offset = 0;
        for next in memchr::Memchr::new(b'\n', mmap) {
            let split = memchr::memchr(b';', &mmap[offset..]).expect("missing ';'");

            let name = unsafe { std::str::from_utf8_unchecked(&mmap[offset..offset + split]) };

            let value = Self::parse_value(&mmap[offset + split + 1..next]);

            if let Some(measurement) = measurements.get_mut(name) {
                measurement.update(value);
            } else {
                measurements.insert(name.to_owned(), Measurement::new(value));
            }

            offset = next + 1;
        }

        measurements
    }

    fn parse_value(buf: &[u8]) -> f64 {
        let mut value = 0.0;

        let mut negative = false;
        let mut dot = false;

        for digit in buf {
            let digit = *digit;
            if digit == b'-' {
                negative = true;
            } else if digit == b'.' {
                dot = true;
            } else {
                let digit = digit - b'0';

                if !dot {
                    value = value * 10.0 + digit as f64;
                } else {
                    value += digit as f64 / 10.0;
                }
            }
        }

        if negative {
            value *= -1.0;
        }

        value
    }
}
