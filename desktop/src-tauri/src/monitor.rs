use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Listener, Runtime};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    None,
    BecameOffline,
    BecameOnline,
}

pub const OFFLINE_THRESHOLD: u32 = 3;
const POLL_INTERVAL: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct MonitorState {
    status: MonitorStatus,
    consecutive_failures: u32,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            status: MonitorStatus::Online,
            consecutive_failures: 0,
        }
    }

    #[allow(dead_code)]
    pub fn status(&self) -> MonitorStatus {
        self.status
    }

    pub fn record_success(&mut self) -> Transition {
        self.consecutive_failures = 0;
        if self.status == MonitorStatus::Offline {
            self.status = MonitorStatus::Online;
            Transition::BecameOnline
        } else {
            Transition::None
        }
    }

    pub fn record_failure(&mut self) -> Transition {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.status == MonitorStatus::Online
            && self.consecutive_failures >= OFFLINE_THRESHOLD
        {
            self.status = MonitorStatus::Offline;
            Transition::BecameOffline
        } else {
            Transition::None
        }
    }
}

impl Default for MonitorState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn spawn<R: Runtime>(app: AppHandle<R>, initial_url: String) {
    let state = Arc::new(Mutex::new(MonitorState::new()));
    let current_url = Arc::new(Mutex::new(initial_url));

    {
        let current_url = current_url.clone();
        app.listen("server-url-changed", move |event| {
            if let Ok(url) = serde_json::from_str::<String>(event.payload()) {
                let current_url = current_url.clone();
                tauri::async_runtime::spawn(async move {
                    *current_url.lock().await = url;
                });
            }
        });
    }

    tauri::async_runtime::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
        {
            Ok(c) => c,
            Err(err) => {
                eprintln!("[monitor] failed to build http client: {err}");
                return;
            }
        };

        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            let url = current_url.lock().await.clone();
            if url.is_empty() {
                continue;
            }
            let probe_url = format!("{}/api/health", url.trim_end_matches('/'));
            let ok = match client.get(&probe_url).send().await {
                Ok(resp) => resp.status().is_success(),
                Err(_) => false,
            };
            let transition = {
                let mut s = state.lock().await;
                if ok {
                    s.record_success()
                } else {
                    s.record_failure()
                }
            };
            match transition {
                Transition::BecameOffline => {
                    let _ = app.emit("monitor-status-changed", "offline");
                }
                Transition::BecameOnline => {
                    let _ = app.emit("monitor-status-changed", "online");
                }
                Transition::None => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_online() {
        let s = MonitorState::new();
        assert_eq!(s.status(), MonitorStatus::Online);
    }

    #[test]
    fn two_failures_do_not_trip_offline() {
        let mut s = MonitorState::new();
        assert_eq!(s.record_failure(), Transition::None);
        assert_eq!(s.record_failure(), Transition::None);
        assert_eq!(s.status(), MonitorStatus::Online);
    }

    #[test]
    fn three_consecutive_failures_go_offline() {
        let mut s = MonitorState::new();
        s.record_failure();
        s.record_failure();
        assert_eq!(s.record_failure(), Transition::BecameOffline);
        assert_eq!(s.status(), MonitorStatus::Offline);
    }

    #[test]
    fn success_resets_failure_counter() {
        let mut s = MonitorState::new();
        s.record_failure();
        s.record_failure();
        s.record_success();
        assert_eq!(s.record_failure(), Transition::None);
        assert_eq!(s.status(), MonitorStatus::Online);
    }

    #[test]
    fn first_success_after_offline_goes_online() {
        let mut s = MonitorState::new();
        s.record_failure();
        s.record_failure();
        s.record_failure();
        assert_eq!(s.record_success(), Transition::BecameOnline);
        assert_eq!(s.status(), MonitorStatus::Online);
    }

    #[test]
    fn repeated_failures_while_offline_emit_no_transition() {
        let mut s = MonitorState::new();
        s.record_failure();
        s.record_failure();
        s.record_failure();
        assert_eq!(s.record_failure(), Transition::None);
    }
}
