use std::process::Command;
use crate::error::LatuiError;

/// Centralized engine for executing various types of commands.
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Executes a command with specified options.
    pub fn spawn(
        program: &str,
        args: &[&str],
        env_vars: &[(&str, &str)],
        detached: bool,
    ) -> Result<(), LatuiError> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        
        for (key, val) in env_vars {
            cmd.env(key, val);
        }

        if detached {
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                unsafe {
                    cmd.pre_exec(|| {
                        libc::setsid();
                        Ok(())
                    });
                }
            }
        }

        cmd.spawn().map_err(|e| LatuiError::Execution {
            command: program.to_string(),
            source: e,
        })?;
        Ok(())
    }

    /// Executes a command via the user's shell.
    pub fn spawn_shell(command: &str, env_vars: &[(&str, &str)]) -> Result<(), LatuiError> {
        tracing::info!("ExecutionEngine: spawning shell command: {}", command);
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Self::spawn(&shell, &["-c", command], env_vars, true)
    }

    /// Executes a desktop application.
    pub fn spawn_desktop_app(exec_path: &str) -> Result<(), LatuiError> {
        tracing::info!("ExecutionEngine: spawning desktop app: {}", exec_path);
        let cleaned_exec = exec_path.split_whitespace().next().unwrap_or(exec_path);
        Self::spawn(cleaned_exec, &[], &[], true)
    }
}
