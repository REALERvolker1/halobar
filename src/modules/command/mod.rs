use std::process::Stdio;

use super::*;

const DEFAULT_SHELL: &str = "/bin/sh";

use smallvec::SmallVec;
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};

config_struct! {
    @config {Clone}
    [Command]
    key: String = String::new(),
    command: String = "date".into(),
    shell: String = String::new(),
    env: AHashMap<String, String> = AHashMap::new(),
    output_type: OutputTypeConfig = OutputTypeConfig::default(),
}

struct CommandBuilder {
    pub output_type: OutputType,
    pub key: String,
    pub command: String,
    pub shell: Option<String>,
    pub env: Vec<(String, String)>,
}
impl CommandBuilder {
    /// Create a [`Comm`] struct out of this builder
    pub async fn run(&mut self, sender: Arc<mpsc::UnboundedSender<String>>) -> CommandResult<()> {
        let mut command = Command::new(self.shell.as_deref().unwrap_or(DEFAULT_SHELL));

        command.args(["-c", self.command.as_str()]);
        command.envs(self.env.iter().map(|s| (s.0.as_str(), s.1.as_str())));

        command.stdout(Stdio::piped());
        command.stdin(Stdio::null());

        match self.output_type {
            OutputType::Static => {
                let output = command.output().await?;

                let stdoutput = String::from_utf8_lossy(&output.stdout);

                for line in stdoutput.lines() {
                    sender.send(line.to_owned())?;
                }
            }
            OutputType::Poll(duration) => loop {
                let (output, _) = try_join!(command.output(), async {
                    // I make a new future like this because try_join expects both futures to return Result
                    tokio::time::sleep(duration).await;
                    Ok(())
                })?;
                let stdoutput = String::from_utf8_lossy(&output.stdout);

                for line in stdoutput.lines() {
                    sender.send(line.to_owned())?;
                }
            },
            OutputType::Watcher => {
                /// This is the newline character (\n)
                const NEWLINE: u8 = 10;
                // TODO: Remove
                debug_assert_eq!(NEWLINE, *b"\n".first().unwrap());

                let mut child = command.spawn()?;

                if let Some(pid) = child.id() {
                    debug!(
                        "Spawned command {} with pid {pid}: {}",
                        &self.key, &self.command
                    );
                }

                let mut child_stdout =
                    child
                        .stdout
                        .take()
                        .ok_or_else(|| match child.start_kill() {
                            Ok(()) => CommandError::MissingStdout,
                            Err(e) => e.into(),
                        })?;

                let mut current_line = Vec::new();
                let mut buffer = [0u8; 256];

                loop {
                    trace!("Reading output from command {}", &self.key);

                    let read_bytes = child_stdout.read(&mut buffer).await?;

                    if read_bytes == 0 {
                        // in this case, the child process is probably dead, since tokio just awaits otherwise.
                        // I will just make sure
                        match child.try_wait()? {
                            Some(status) => {
                                error!("Command {} exited with status {}", self.key, status);
                                break;
                            }
                            None => {
                                // No bytes were read, this should not occur!
                                error!("Command {} exited without notifying watcher!", self.key);
                                break;
                            }
                        }
                    }

                    let mut buffer_iter = buffer.iter_mut();

                    while let Some(byte) = buffer_iter.next() {
                        // if it is a nullbyte, we are probably waiting on some more bytes to be read
                        match *byte {
                            0 => break,
                            NEWLINE => {
                                let line_string = String::from_utf8_lossy(&current_line);
                                sender.send(line_string.into())?;

                                current_line.clear();
                            }
                            _ => {
                                current_line.push(*byte);
                            }
                        }

                        // it is safe to reuse the same buffer because I am clearing all the data after reading it.
                        *byte = 0;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct CommandModule;
impl ModuleDataProvider for CommandModule {
    type ServerConfig = CommandConfig;
    async fn main(
        config: Self::ServerConfig,
        requests: Vec<DataRequest>,
        yield_channel: mpsc::UnboundedSender<ModuleYield>,
    ) -> R<()> {
        todo!();
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum OutputTypeConfig {
    #[default]
    Static,
    Watcher,
    Poll(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Static,
    Watcher,
    Poll(Duration),
}
impl From<OutputTypeConfig> for OutputType {
    fn from(value: OutputTypeConfig) -> Self {
        match value {
            OutputTypeConfig::Static => Self::Static,
            OutputTypeConfig::Watcher => Self::Watcher,
            OutputTypeConfig::Poll(s) => Self::Poll(Duration::from_secs(s)),
        }
    }
}
// impl OutputType {
//     /// Run a command based on the output type.
//     ///
//     /// This could return at any moment, the caller is responsible for ensuring everything works as intended.
//     pub async fn run(&self, command: RunningCommand) -> R<Option<String>> {
//         match self {
//             Self::Static => command.ou,
//         }
//     }
// }

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("(0)")]
    Io(#[from] tokio::io::Error),
    #[error("Child stdout is missing!")]
    MissingStdout,
    #[error("Error sending data through channel")]
    SendError,
}
impl<T> From<mpsc::error::SendError<T>> for CommandError {
    fn from(_value: mpsc::error::SendError<T>) -> Self {
        Self::SendError
    }
}
type CommandResult<T> = std::result::Result<T, CommandError>;
