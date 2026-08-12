use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::AppError;

pub const SCRIPT_SCHEMA: &str = "aetherion.script/v1";
pub const REPORT_SCHEMA: &str = "aetherion.script-report/v1";
const HARD_MAX_COMMANDS: u64 = 10_000;
const HARD_MAX_TICKS: u64 = 1_000_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script {
    pub schema: String,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    pub commands: Vec<CommandLine>,
    pub budget: Budget,
    pub on_error: OnError,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CommandLine {
    Text(String),
    Args(Vec<String>),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub max_commands: u64,
    pub max_ticks_total: u64,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    Stop,
    Continue,
}
#[derive(Debug, Serialize)]
pub struct ScriptReport {
    pub schema: &'static str,
    pub results: Vec<CommandResult>,
    pub ok: bool,
    pub commands_consumed: u64,
    pub ticks_consumed: u64,
}
#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub index: usize,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub ok: bool,
}

pub fn run(path: &Path, dry_run: bool, report_path: Option<&Path>) -> Result<ScriptReport> {
    let data = fs::read(path).map_err(|e| format!("script_read: {}: {e}", path.display()))?;
    let script: Script =
        serde_json::from_slice(&data).map_err(|e| format!("script_invalid: {e}"))?;
    if script.schema != SCRIPT_SCHEMA {
        return Err("script_version".into());
    }
    if script.budget.max_commands > HARD_MAX_COMMANDS
        || script.budget.max_ticks_total > HARD_MAX_TICKS
    {
        return Err(AppError::new("script_budget_hard_limit").with_exit_code(3));
    }
    if script.commands.len() as u64 > script.budget.max_commands {
        return Err(AppError::new("script_budget_commands").with_exit_code(3));
    }
    let mut results = Vec::new();
    let mut ticks = 0_u64;
    for (index, command) in script.commands.iter().enumerate() {
        if ticks >= script.budget.max_ticks_total {
            return Err(AppError::new("script_budget_ticks").with_exit_code(3));
        }
        let args = substituted(command, &script.vars)?;
        let exit_code = if dry_run { 0 } else { execute_command(&args) };
        let ok = exit_code == 0;
        ticks += 1;
        results.push(CommandResult {
            index,
            args,
            exit_code,
            ok,
        });
        if !ok && script.on_error == OnError::Stop {
            break;
        }
    }
    let ok = results.iter().all(|r| r.ok) && results.len() == script.commands.len();
    let report = ScriptReport {
        schema: REPORT_SCHEMA,
        commands_consumed: results.len() as u64,
        ticks_consumed: ticks,
        results,
        ok,
    };
    if let Some(report_path) = report_path {
        write_atomic(
            report_path,
            &serde_json::to_vec_pretty(&report)
                .map_err(|e| format!("script_report_serialize: {e}"))?,
        )?;
    }
    if !report.ok {
        let json = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("script_report_serialize: {e}"))?;
        return Err(AppError::outcome("script_failed", 1, json));
    }
    Ok(report)
}

fn substituted(line: &CommandLine, vars: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let items: Vec<String> = match line {
        CommandLine::Text(text) => text.split_whitespace().map(str::to_owned).collect(),
        CommandLine::Args(args) => args.clone(),
    };
    let result: Vec<String> = items
        .into_iter()
        .map(|item| substitute(&item, vars))
        .collect::<Result<_>>()?;
    if result.is_empty() {
        return Err("script_command_empty".into());
    }
    Ok(result)
}
fn substitute(value: &str, vars: &BTreeMap<String, String>) -> Result<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 2..];
        let end = tail.find("}}").ok_or("script_variable_unclosed")?;
        let key = &tail[..end];
        out.push_str(
            vars.get(key)
                .ok_or_else(|| format!("script_variable_missing: {key}"))?,
        );
        rest = &tail[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}
fn execute_command(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("true") => 0,
        Some("false") => 1,
        Some("echo") | Some("noop") => 0,
        _ => 2,
    }
}
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| format!("script_report_mkdir: {}: {e}", parent.display()))?;
    }
    let mut tmp = PathBuf::from(path);
    tmp.set_extension(format!("tmp-{}", std::process::id()));
    let mut value = bytes.to_vec();
    value.push(b'\n');
    fs::write(&tmp, value).map_err(|e| format!("script_report_write: {}: {e}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("script_report_replace: {}: {e}", path.display()))?;
    }
    fs::rename(&tmp, path)
        .map_err(|e| format!("script_report_rename: {}: {e}", path.display()).into())
}
