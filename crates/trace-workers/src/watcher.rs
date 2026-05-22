use std::{path::PathBuf, time::Duration};

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use trace_services::events::CoreEvent;

pub struct Watcher {
    vault_path: PathBuf,
    tx: broadcast::Sender<CoreEvent>,
}

impl Watcher {
    pub fn new(vault_path: PathBuf, tx: broadcast::Sender<CoreEvent>) -> Self {
        Self { vault_path, tx }
    }

    pub async fn run(&self) {
        let tx = self.tx.clone();
        let vault_path = self.vault_path.clone();

        let result = tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                // Pass the mpsc sender directly — it implements DebounceEventHandler.
                let (dtx, drx) = std::sync::mpsc::channel::<DebounceEventResult>();

                let mut debouncer = new_debouncer(Duration::from_millis(300), dtx)?;
                debouncer.watcher().watch(&vault_path, RecursiveMode::Recursive)?;
                info!("watcher: watching {:?}", vault_path);

                for res in drx {
                    match res {
                        Ok(events) => {
                            for ev in events {
                                let path = ev.path.to_string_lossy().into_owned();
                                let kind = format!("{:?}", ev.kind);
                                info!("watcher: changed: {path} [{kind}]");
                                let _ = tx.send(CoreEvent::FileChanged { path, kind });
                            }
                        }
                        Err(e) => warn!("watcher: {e}"),
                    }
                }

                Ok(())
            },
        )
        .await;

        match result {
            Err(e) => tracing::error!("watcher: task panic: {e}"),
            Ok(Err(e)) => tracing::error!("watcher: {e}"),
            Ok(Ok(())) => {}
        }
    }
}
