// SPDX-License-Identifier: MPL-2.0
//! Retry only an unopened Windows pipe, never a handshake or a product request.
use anyhow::{Context, Result};
use std::{io, time::Duration};

// Match the Win32 code, not ErrorKind (whose mapping differs on test hosts).
const ERROR_PIPE_BUSY: i32 = 231;
const RETRY_DELAY: Duration = Duration::from_millis(10);

pub(super) async fn connect<T>(
    mut open: impl FnMut() -> io::Result<T>,
    timeout: Duration,
) -> Result<T> {
    tokio::time::timeout(timeout, async {
        loop {
            match open() {
                // The server replenishes its listening instance after accepting
                // a client. Concurrent callers can reach it in that short gap.
                Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                result => return result,
            }
        }
    })
    .await
    .context("Regressee's background engine stayed busy. Wait a moment and try again.")?
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(test: impl std::future::Future<Output = Result<()>>) -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(test)
    }

    #[test]
    fn ready_pipe_is_opened_once() -> Result<()> {
        run(async {
            let mut attempts = 0;
            let value = connect(
                || {
                    attempts += 1;
                    Ok(42)
                },
                Duration::from_secs(1),
            )
            .await?;
            assert_eq!(value, 42);
            assert_eq!(attempts, 1);
            Ok(())
        })
    }

    #[test]
    fn busy_pipe_is_retried_until_open_without_replaying_success() -> Result<()> {
        run(async {
            let mut attempts = 0;
            let value = connect(
                || {
                    attempts += 1;
                    if attempts < 3 {
                        Err(io::Error::from_raw_os_error(ERROR_PIPE_BUSY))
                    } else {
                        Ok(42)
                    }
                },
                Duration::from_secs(1),
            )
            .await?;
            assert_eq!(value, 42);
            assert_eq!(attempts, 3);
            Ok(())
        })
    }

    #[test]
    fn other_errors_are_not_retried_or_replaced() -> Result<()> {
        run(async {
            // Windows: missing endpoint, access denied, disconnected pipe.
            for code in [2, 5, 233] {
                let mut attempts = 0;
                let error = connect::<()>(
                    || {
                        attempts += 1;
                        Err(io::Error::from_raw_os_error(code))
                    },
                    Duration::from_secs(1),
                )
                .await
                .unwrap_err();
                assert_eq!(attempts, 1);
                assert_eq!(
                    error.downcast_ref::<io::Error>().unwrap().raw_os_error(),
                    Some(code)
                );
            }
            Ok(())
        })
    }

    #[test]
    fn permanent_error_after_busy_stops_retrying() -> Result<()> {
        run(async {
            let mut attempts = 0;
            let error = connect::<()>(
                || {
                    attempts += 1;
                    Err(io::Error::from_raw_os_error(if attempts == 1 {
                        ERROR_PIPE_BUSY
                    } else {
                        5
                    }))
                },
                Duration::from_secs(1),
            )
            .await
            .unwrap_err();
            assert_eq!(attempts, 2);
            assert_eq!(
                error.downcast_ref::<io::Error>().unwrap().raw_os_error(),
                Some(5)
            );
            Ok(())
        })
    }

    #[test]
    fn persistently_busy_pipe_has_a_deadline_and_actionable_error() -> Result<()> {
        run(async {
            let mut attempts = 0;
            let error = connect::<()>(
                || {
                    attempts += 1;
                    Err(io::Error::from_raw_os_error(ERROR_PIPE_BUSY))
                },
                Duration::from_millis(50),
            )
            .await
            .unwrap_err();
            assert!(attempts >= 1);
            assert!(
                error
                    .downcast_ref::<tokio::time::error::Elapsed>()
                    .is_some()
            );
            let message = error.to_string();
            assert!(message.contains("background engine stayed busy"));
            assert!(message.contains("try again"));
            assert!(!message.contains("os error"));
            Ok(())
        })
    }
}
