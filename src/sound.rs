use rodio::{
    source::{SineWave, Source},
    OutputStream, Sink,
};
use std::{
    collections::VecDeque,
    process::{Child, Command},
    time::Instant,
};
use std::{thread, time::Duration};

use crate::Row;

const RATE_WPM: &str = "300";

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

    // pub fn play(&self) {
    //     let mut command = Command::new("afplay");
    //     command.arg("-r").arg("44100");
    //     command.arg("-c").arg("2");
    //     command.arg("-t").arg(&format!("{}", self.duration));
    //     command.arg("-v").arg(&format!("{}", self.volume));
    //     command.arg("-f").arg("s16le");
    //     command.arg("-");
    //     let mut command = command.stdin(Stdio::piped()).spawn().unwrap();
    //     let stdin = command.stdin.as_mut().unwrap();
    //     let mut buffer = [0.0f32; 44100];
    // }
}

impl Audible for Tone {
    fn play_and_wait(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let mut source = SineWave::new(self.frequency)
            .amplify(self.volume)
            .take_duration(Duration::from_secs_f32(self.duration));

        source.set_filter_fadeout();

        sink.append(source);
        // The sound plays in a separate thread. This call will block the current thread until the sink
        // has finished playing all its queued sounds.
        sink.sleep_until_end();
    }

    fn play(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let mut source = SineWave::new(self.frequency)
            .amplify(self.volume)
            .take_duration(Duration::from_secs_f32(self.duration));

        source.set_filter_fadeout();

        sink.append(source);
    }

    fn stop(&self) {}
}

/// An Utterance is a spoken phrase.
pub struct Utterance {
    text: String,
}

impl Utterance {
    /// Create a new Utterance.
    ///
    /// # Arguments
    ///
    /// * `text` - The text of the utterance.
    ///
    /// # Returns
    ///
    /// A new Utterance.
    ///
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// Speak the utterance and wait for the speech to finish.
    pub fn speak_and_wait(&self) {
        let mut command = Command::new("say");
        command.arg("-r").arg(RATE_WPM);
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
        command.arg("-r").arg(RATE_WPM);
        command.arg(&self.text);
        command.spawn().unwrap()
    }

    /// Create a copy of the utterance.
    ///
    /// # Returns
    ///
    /// A copy of the utterance.
    ///
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
        }
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

/// An UtteranceManager manages a queue of Utterances and handles
/// requests to speak the next utterance. Utterances can be added to the
/// front or the back of the queue (depending on priority), and callers can
/// also clear the queue.
/// The utterances are spoken by the system's speech synthesizer in a separate
/// thread, which is handled by the crate `std::thread`. There is a mutex
/// guarding the queue, so the queue can be accessed by multiple threads and
/// so that we can cancel the speech synthesis with `stop`.
/// The utterance manager is also responsible for keeping track of the
/// current utterance, so that it can be cancelled if the user requests it.
#[derive(Default)]
struct UtteranceManager {
    queue: VecDeque<Utterance>,
    current_utterance: Option<Utterance>,
    current_utterance_start: Option<Instant>,
    current_child_process: Option<Child>,
}

impl UtteranceManager {
    /// Create a new UtteranceManager.
    /// # Returns
    /// A new UtteranceManager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an utterance to the front of the queue.
    ///
    /// # Arguments
    ///
    /// * `utterance` - The utterance to add.
    ///
    pub fn say_next(&mut self, utterance: Utterance) {
        self.queue.push_front(utterance);
    }

    /// Add an utterance to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `utterance` - The utterance to add.
    ///
    /// # Returns
    ///
    /// None
    ///
    pub fn say(&mut self, utterance: Utterance) {
        self.queue.push_back(utterance);
    }

    /// Clear the queue.
    ///
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Spawn a thread to speak the next utterance in the queue, and keep
    /// track of the utterance so that it can be cancelled if the user requests
    /// it.
    pub fn speak_next_or_wait(&mut self) {
        if let Some(utterance) = self.queue.pop_front() {
            self.current_utterance = Some(utterance.clone());
            let utterance = utterance.clone();
            self.current_child_process = Some(utterance.speak());
            self.current_utterance_start = Some(Instant::now());
        } else {
            self.current_utterance = None;
            self.current_child_process = None;
        }
    }

    /// Cancel the current utterance, if there is one. This is done by
    /// killing the child thread that is speaking the utterance.
    /// If there is no current utterance, this is a no-op.
    pub fn kill(&mut self) {
        if let Some(child_process) = &mut self.current_child_process {
            child_process.kill().unwrap();
        }
        self.current_utterance = None;
        self.current_child_process = None;
    }

    /// Interrupts the current utterance, if there is one. This is done by
    /// killing the child thread that is speaking the utterance.
    ///
    /// # Arguments
    ///
    /// * `interrupt_utterance` - The utterance to interrupt with.
    ///
    /// # Returns
    ///
    /// None
    ///
    pub fn interrupt_and_say(&mut self, interrupt_utterance: Utterance) {
        self.kill();
        self.say_next(interrupt_utterance);
    }

    /// Clears the queue and adds an utterance to the back of the queue.
    ///
    /// # Arguments
    ///
    /// * `utterance` - The utterance to add.
    ///
    /// # Returns
    ///
    /// None
    ///
    pub fn clear_and_say(&mut self, utterance: Utterance) {
        self.clear();
        self.say_next(utterance);
    }

    /// Speaks an utterance synchronously.
    ///
    /// # Arguments
    ///
    /// * `utterance` - The utterance to speak.
    ///
    /// # Returns
    ///
    /// None
    pub fn say_and_wait(&mut self, utterance: Utterance) {
        utterance.speak_and_wait();
    }
}

impl Audible for Utterance {
    fn play_and_wait(&self) {
        self.speak_and_wait();
    }

    fn play(&self) {
        self.speak();
    }

    fn stop(&self) {}
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

    pub fn play_next(&mut self, sound: Box<dyn Audible>) {
        self.queue.push_front(sound);
    }

    pub fn play(&mut self, sound: Box<dyn Audible>) {
        self.queue.push_back(sound);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn speak_next_or_wait(&mut self) {
        while let Some(sound) = self.queue.pop_front() {
            sound.as_ref().play_and_wait();
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
        self.play_next(interrupt_sound);
    }

    pub fn clear_and_play(&mut self, sound: Box<dyn Audible>) {
        self.clear();
        self.play_next(sound);
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
