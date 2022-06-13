use std::{
    collections::VecDeque,
    process::{Child, Command},
    time::Instant,
};

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
pub struct UtteranceManager {
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
    pub fn stop(&mut self) {
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
        self.stop();
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
