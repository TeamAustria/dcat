use std::fs::File;
use std::io::{Read, stdin, stdout, Write};
use std::thread;
use std::time::Duration;

use clap::Parser;
use delay_queue::{Delay, DelayQueue};

fn main() {
    let args = Args::parse();

    let mut queue: DelayQueue<Delay<Vec<u8>>> = DelayQueue::new();

    let mut consumer_queue = queue.clone();
    let consumer_handle = thread::spawn(move || {
        let mut writer = stdout();

        loop {
            let item = consumer_queue.pop();
            let value = item.value;
            if value.is_empty() {
                break;
            }
            writer.write_all(value.as_slice()).unwrap();
        }
    });

    let mut readers: Vec<Box<dyn Read>> = args.input_files
        .unwrap_or(vec!["-".to_string()])
        .iter()
        .map(|file| match file.as_str() {
            "-" => Box::new(stdin()) as Box<dyn Read>,
            _ => Box::new(File::open(file).expect("Failed to open file")),
        })
        .collect();

    let mut buffer = vec![0u8; args.buffer_size as usize];

    for reader in readers.iter_mut() {
        loop {
            let read = reader.read(&mut buffer).unwrap();
            if read == 0 {
                break;
            }
            queue.push(Delay::for_duration(
                buffer[0..read].to_vec(),
                Duration::from_millis(args.delay_ms),
            ));
        }
    }
    queue.push(Delay::for_duration(
        vec![],
        Duration::from_millis(args.delay_ms)
    ));

    consumer_handle.join().unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1000)]
    delay_ms: u64,
    #[arg(short, long, default_value_t = 1024 * 1024)]
    buffer_size: u64,
    input_files: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::process::Stdio;

    use rand::RngCore;

    fn generate_test_data() -> Vec<u8> {
        let mut data = [0u8; 1024];
        rand::thread_rng().fill_bytes(&mut data);
        data.to_vec()
    }

    #[test]
    fn basic_in_out() {
        let data = generate_test_data();

        let process = test_bin::get_test_bin("dcat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let mut writer = process.stdin.as_ref().unwrap();
            writer.write_all(&data).unwrap();
            writer.flush().unwrap();
        }

        let output = process.wait_with_output().unwrap().stdout;

        assert_eq!(data, output);
    }
}
