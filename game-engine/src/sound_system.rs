use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Cursor};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct SoundSystem {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    cache: Mutex<HashMap<String, Arc<Vec<u8>>>>,
}

impl SoundSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(SoundSystem {
            _stream: stream,
            stream_handle,
            cache: Mutex::new(HashMap::new()),
        })
    }
    pub fn play_sound(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sound_data = {
            let mut cache = self.cache.lock().unwrap();
            cache
                .entry(path.to_owned())
                .or_insert_with(|| {
                    let bytes = fs::read(path).unwrap_or_else(|err| {
                        panic!("Error al leer el archivo {}: {:?}", path, err)
                    });
                    Arc::new(bytes)
                })
                .clone()
        };

        let stream_handle = self.stream_handle.clone();
        thread::spawn(move || {
            let cursor = Cursor::new(sound_data.as_ref().to_owned());
            let decoder = Decoder::new(BufReader::new(cursor)).unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(decoder);
            sink.sleep_until_end();
        });

        Ok(())
    }
}
