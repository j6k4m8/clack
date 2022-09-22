use std::{
    collections::VecDeque,
    process::{Child, Command},
    time::{Duration, Instant},
};

use rodio::{source::SineWave, OutputStream, Sink, Source};

use crate::Row;

pub const SCALE_NOTES_MAP: &[f32] = &[
    262.0, /* C  */
    277.0, /* C# */
    294.0, /* D  */
    311.0, /* D# */
    330.0, /* E  */
    349.0, /* F  */
    370.0, /* F# */
    392.0, /* G  */
    415.0, /* G# */
    440.0, /* A  */
    466.0, /* A# */
    494.0, /* B  */
];

pub const PENTATONIC_SCALE: &[f32] = &[
    SCALE_NOTES_MAP[1],  /* C# */
    SCALE_NOTES_MAP[3],  /* D# */
    SCALE_NOTES_MAP[6],  /* F# */
    SCALE_NOTES_MAP[8],  /* G# */
    SCALE_NOTES_MAP[10], /* A# */
];

/// A trait for objects that can be played by the sound system.
/// This is used to abstract away the underlying sound players.
pub trait Audible {
    /// Start playing the sound.
    fn play(&self);

    /// Play the sound and wait for it to finish.
    fn play_and_wait(&self);
}

/// A trait for Audibles that can be cancelled.
pub trait CancellableAudible: Audible {
    /// Stop playing the sound.
    fn stop(&self);
}

#[derive(Clone, Copy)]
pub struct Tone {
    pub frequency: f32,
    pub duration: f32,
    pub volume: f32,
}

impl Tone {
    pub fn new(frequency: f32, duration: f32, volume: f32) -> Self {
        Self {
            frequency,
            duration,
            volume,
        }
    }
}

impl Audible for Tone {
    fn play(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let mut source = SineWave::new(self.frequency)
            .amplify(self.volume)
            .take_duration(Duration::from_secs_f32(self.duration));

        source.set_filter_fadeout();

        sink.append(source);
    }

    fn play_and_wait(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let mut source = SineWave::new(self.frequency)
            .amplify(self.volume)
            .take_duration(Duration::from_secs_f32(self.duration));

        source.set_filter_fadeout();

        sink.append(source);
        sink.sleep_until_end();
    }
}

/// An Utterance is a spoken phrase.
#[derive(Clone)]
pub struct Utterance {
    text: String,
    rate_wpm: i64,
}

impl Utterance {
    /// Create a new Utterance.
    ///
    /// # Arguments
    ///
    /// * `text` - The text of the utterance.
    /// * `rate_wpm` - The rate of the utterance in words per minute.
    ///
    /// # Returns
    ///
    /// A new Utterance.
    ///
    pub fn new(text: String) -> Self {
        Self {
            text,
            rate_wpm: 300,
        }
    }

    pub fn from_text_and_wpm(text: String, rate_wpm: i64) -> Self {
        Self { text, rate_wpm }
    }

    /// Speak the utterance and wait for the speech to finish.
    pub fn speak_and_wait(&self) {
        let mut command = Command::new("say");
        command.arg("-r").arg(self.rate_wpm.to_string());
        command.arg(&self.text);
        command.output().unwrap();
    }

    /// Speak the utterance and return a Child of the subprocess.
    ///
    /// # Returns
    ///
    /// A Child of the subprocess.
    ///
    pub fn speak(&self) -> Child {
        let mut command = Command::new("say");
        command.arg("-r").arg(self.rate_wpm.to_string());
        command.arg(&self.text);
        command.spawn().unwrap()
    }
}

impl From<&str> for Utterance {
    /// Create a new Utterance from a string.
    ///
    /// # Arguments
    ///
    /// * `text` - The text of the utterance.
    ///
    /// # Returns
    ///
    /// A new Utterance.
    fn from(text: &str) -> Self {
        Self::new(String::from(text))
    }
}

impl Audible for Utterance {
    fn play_and_wait(&self) {
        self.speak_and_wait();
    }

    fn play(&self) {
        self.speak();
    }
}

/// A sequence of Audibles that are played sequentially:
pub struct SoundSequence {
    audibles: Vec<Box<dyn Audible>>,
}

impl SoundSequence {
    /// Create a new SoundSequence.
    ///
    /// # Arguments
    ///
    /// * `audibles` - The audibles to play.
    ///
    /// # Returns
    ///
    /// A new SoundSequence.
    ///
    pub fn new(audibles: Vec<Box<dyn Audible>>) -> Self {
        Self { audibles }
    }
}

impl Audible for SoundSequence {
    fn play(&self) {
        todo!()
    }

    fn play_and_wait(&self) {
        for audible in &self.audibles {
            audible.play_and_wait();
        }
    }
}
pub struct SoundManager {
    queue: VecDeque<Box<dyn Audible>>,
    current_sound: Option<Box<dyn Audible>>,
    current_sound_start: Option<Instant>,
    current_child_process: Option<Child>,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current_sound: None,
            current_sound_start: None,
            current_child_process: None,
        }
    }

    pub fn prepend(&mut self, sound: Box<dyn Audible>) {
        self.queue.push_front(sound);
    }

    pub fn append(&mut self, sound: Box<dyn Audible>) {
        self.queue.push_back(sound);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn play_next_or_wait(&mut self) {
        while let Some(sound) = self.queue.pop_front() {
            sound.as_ref().play_and_wait();
            self.current_sound = Some(sound);
            self.current_sound_start = Some(Instant::now());
        }
        self.current_sound = None;
        self.current_child_process = None;
    }

    pub fn kill(&mut self) {
        if let Some(child_process) = &mut self.current_child_process {
            child_process.kill().unwrap();
        }
        self.current_sound = None;
        self.current_child_process = None;
    }

    pub fn interrupt_and_play(&mut self, interrupt_sound: Box<dyn Audible>) {
        self.kill();
        self.prepend(interrupt_sound);
    }

    pub fn clear_and_play(&mut self, sound: Box<dyn Audible>) {
        self.clear();
        self.prepend(sound);
    }

    pub fn play_and_wait(&mut self, sound: Box<dyn Audible>) {
        sound.play_and_wait();
    }

    pub fn play_row(&mut self, row: &Row) {
        row.play(self);
    }

    pub fn play_row_and_wait(&mut self, row: Row) {
        row.play_blocking(self);
    }
}
