use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use runvault::{Run, RunOptions};
use runvault_socsim::{RunHandle, RunvaultRecorder};
use serde_json::Value;

use crate::analyze::{condition_label, ConditionAggregate};
use crate::runner::TrialResult;

pub type RecordResult<T> = Result<T, Box<dyn Error>>;

pub fn start_run(subcommand: &str, parameters: &Value, label: &str, scratch: bool) -> RecordResult<(RunvaultRecorder, RunHandle)> {
    let run = Run::start(
        RunOptions::new("rumor", subcommand)
            .repo_id("rumor-forgetting")
            .domain("simulation")
            .scratch(scratch)
            .parameters(parameters)?
            .label(label)
            .seed_pointers(["/seeds"])
            .master_seed(0),
    )?;
    let recorder = RunvaultRecorder::new(run).step_unit("time").scope("run");
    let handle = recorder.handle();
    Ok((recorder, handle))
}

pub fn write_run_file(handle: &RunHandle, relative: &str, contents: &str) -> RecordResult<PathBuf> {
    let run_dir = handle.dir().ok_or("run is already closed")?;
    let path = run_dir.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    Ok(path)
}

pub fn record_results(handle: &RunHandle, trials: &[TrialResult], aggregates: &[ConditionAggregate]) -> RecordResult<()> {
    let mut metrics = Vec::new();
    for aggregate in aggregates {
        let cond = condition_label(aggregate.lambda, aggregate.delta);
        metrics.push((format!("p_survive.{cond}"), aggregate.p_survive));
        metrics.push((format!("ever.{cond}"), aggregate.ever));
        metrics.push((format!("ignorant_final.{cond}"), aggregate.ignorant_final));
        if let Some(value) = aggregate.s_qs {
            metrics.push((format!("s_qs.{cond}"), value));
        }
        if let Some(value) = aggregate.known_qs {
            metrics.push((format!("known_qs.{cond}"), value));
        }
    }
    let isolated = trials.iter().map(|trial| trial.isolated_fraction).sum::<f64>() / trials.len() as f64;
    metrics.push(("isolated_frac".to_owned(), isolated));
    let borrowed: Vec<_> = metrics.iter().map(|(name, value)| (name.as_str(), *value)).collect();
    handle.with_run(|run| run.log_metrics("run", &borrowed))??;
    write_csv(handle, "artifacts/trials.csv", trials)?;
    write_csv(handle, "artifacts/aggregates.csv", aggregates)?;
    write_representative_series(handle, trials)?;
    Ok(())
}

fn write_csv<T: serde::Serialize>(handle: &RunHandle, relative: &str, rows: &[T]) -> RecordResult<()> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    for row in rows {
        writer.serialize(row)?;
    }
    let bytes = writer.into_inner()?;
    write_run_file(handle, relative, std::str::from_utf8(&bytes)?)?;
    Ok(())
}

fn write_representative_series(handle: &RunHandle, trials: &[TrialResult]) -> RecordResult<()> {
    let path = handle.dir().ok_or("run is already closed")?.join("artifacts/timeseries.csv");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["lambda", "sigma", "delta", "seed", "t", "i", "s", "r", "ever"])?;
    for trial in trials {
        for sample in &trial.history {
            writer.serialize((
                trial.lambda,
                trial.sigma,
                trial.delta,
                trial.seed,
                sample.t,
                sample.i,
                sample.s,
                sample.r,
                sample.ever,
            ))?;
        }
    }
    writer.flush()?;
    Ok(())
}

pub fn finish(handle: RunHandle) -> RecordResult<PathBuf> {
    Ok(handle.finish()?)
}

pub fn fail(handle: RunHandle, error: &dyn std::fmt::Display) {
    let _ = handle.fail("simulation", error.to_string());
}

pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
