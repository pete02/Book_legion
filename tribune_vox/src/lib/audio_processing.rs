
use reqwest::blocking::Client;
use std::{fs::File, time::Duration};
use serde_json::json;
use hound::{SampleFormat, WavReader,  WavWriter};
use std::io::Cursor;
use std::thread::sleep;




pub fn text_to_wav(
    text: &str,
    voice: &str,
    ip: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    const MAX_RETRIES: usize = 3;
    println!("ip:{}", ip);
    for attempt in 1..=MAX_RETRIES {
        let result = client
            .post(ip)
            .json(&json!({
                "text": text,
                "voice": voice,
                "cfg_weight": "0.4",
                "temperature": "0.9"
            }))
            .send();
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    let wav_bytes = response.bytes()?.to_vec();
                    return Ok(wav_bytes);
                } else {
                    if attempt == MAX_RETRIES {
                        return Err(format!(
                            "Request failed after {} attempts, last status: {}",
                            attempt,
                            response.status()
                        ).into());
                    }
                }
            }
            Err(err) => {
                if attempt == MAX_RETRIES {
                    return Err(format!(
                        "Request failed after {} attempts: {}",
                        attempt, err
                    ).into());
                }
            }
        }

        // Exponential backoff: 500ms, 1s, 2s
        let backoff = Duration::from_millis(500 * (1 << (attempt - 1)));
        sleep(backoff);
    }

    unreachable!()
}

fn get_dummy_reader(ip:&str)->Result<WavReader<Cursor<Vec<u8>>>, Box< dyn std::error::Error>>{
    let dummy_bytes = text_to_wav("Hello", "sofia",ip)?;
    create_reader(dummy_bytes)
}


pub fn create_writer(file:&str,ip:&str)->Result<WavWriter<std::fs::File>,Box<dyn std::error::Error>>{
    let mut new_spec = get_dummy_reader(ip)?.spec();
    new_spec.sample_format = SampleFormat::Int;
    new_spec.bits_per_sample = 16;
    Ok(WavWriter::new(File::create(file)?,new_spec)?)
}


pub fn create_reader(wav_bytes: Vec<u8>)->Result<WavReader<Cursor<Vec<u8>>>, Box<dyn std::error::Error>>{
    Ok(WavReader::new(Cursor::new(wav_bytes))?)
}


pub fn write_samples_to_wav(reader:&mut WavReader<Cursor<Vec<u8>>>, writer:&mut WavWriter<std::fs::File>)->Result<f32,Box<dyn std::error::Error>>{
    let mut num_samples = 0.0;
    let spec=reader.spec();
    match spec.sample_format {
        SampleFormat::Int => {
            for sample in reader.samples::<i16>() {
                writer.write_sample(sample?)?;
                num_samples+=1.0;
            }
        }
        SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                let s = sample?;
                let int_sample =
                    (s * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                writer.write_sample(int_sample)?;
                num_samples+=1.0;
            }
        }
    }
    Ok(num_samples/spec.sample_rate as f32)
}


