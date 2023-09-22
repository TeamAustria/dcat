use std::io::{Read, stdin, stdout, Write};
use std::thread;
use std::time::Duration;
use delay_queue::{Delay, DelayQueue};

fn main() {
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

    let mut reader = stdin();
    let mut buffer = [0; 1024];

    loop {
        let read = reader.read(&mut buffer).unwrap();
        queue.push(Delay::for_duration(
            buffer[0..read].to_vec(),
            Duration::from_secs(1),
        ));
        if read == 0 {
            break;
        }
    }

    consumer_handle.join().unwrap();
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
