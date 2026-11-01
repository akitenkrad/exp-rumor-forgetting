use std::error::Error;
use std::time::Instant;

use clap::{Parser, Subcommand};
use rumor_simulation::analyze::aggregate_conditions;
use rumor_simulation::checks::{run_checks_with_scale, CheckScale};
use rumor_simulation::config::{parse_seed_filter, ExperimentConfig};
use rumor_simulation::record::{fail, finish, record_results, start_run, write_run_file};
use rumor_simulation::runner::run_specs_parallel;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "rumor", about = "Continuous-time I/S/R rumor simulation with forgetting")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run every parameter cell declared by a TOML config.
    Run {
        #[arg(long)]
        config: String,
        /// Comma-separated subset of config seeds, for example `0` or `0,1`.
        #[arg(long)]
        seeds: Option<String>,
        /// Override the run label; defaults to the config name.
        #[arg(long)]
        name: Option<String>,
        /// Mark this run as scratch data.
        #[arg(long)]
        scratch: bool,
        /// Worker threads; defaults to available parallelism.
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Run validations V-1 through V-7 at N=10,000.
    Check {
        /// Mark this run as scratch data.
        #[arg(long)]
        scratch: bool,
    },
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Command::Run {
            config,
            seeds,
            name,
            scratch,
            threads,
        } => run_command(&config, seeds.as_deref(), name.as_deref(), scratch, threads),
        Command::Check { scratch } => check_command(scratch),
    }
}

fn run_command(
    config_path: &str,
    seed_filter: Option<&str>,
    name: Option<&str>,
    scratch: bool,
    threads: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let config = ExperimentConfig::load(config_path)?;
    let selected = seed_filter.map(parse_seed_filter).transpose()?;
    let specs = config.trial_specs(selected.as_deref())?;
    let selected_seeds: Vec<_> = selected.unwrap_or_else(|| config.seeds.clone());
    let label = name.unwrap_or(&config.name);
    let parameters = json!({
        "config": config_path, "name": label, "network": config.network,
        "n": config.n, "mean_degree": config.mean_degree, "sigma": config.sigma,
        "lambdas": config.lambdas, "deltas": config.deltas, "t_max": config.t_max,
        "seeds": selected_seeds, "forget": config.forget,
    });
    let (_recorder, handle) = start_run("run", &parameters, label, scratch)?;
    let started = Instant::now();
    let thread_count = threads.unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from));
    let outcome = (|| -> Result<_, Box<dyn Error + Send + Sync>> {
        let trials = run_specs_parallel(&specs, thread_count)?;
        let aggregates = aggregate_conditions(&trials);
        Ok((trials, aggregates))
    })();
    match outcome {
        Ok((trials, aggregates)) => {
            if let Err(error) = record_results(&handle, &trials, &aggregates) {
                fail(handle, error.as_ref());
                return Err(error);
            }
            let elapsed = started.elapsed();
            write_run_file(
                &handle,
                "logs/timing.txt",
                &format!(
                    "trials={}\nthreads={}\nelapsed_seconds={:.6}\n",
                    trials.len(),
                    thread_count,
                    elapsed.as_secs_f64()
                ),
            )?;
            let directory = finish(handle)?;
            println!("Completed {} trials in {:.3} s", trials.len(), elapsed.as_secs_f64());
            println!("Run directory: {}", directory.display());
            Ok(())
        }
        Err(error) => {
            fail(handle, error.as_ref());
            Err(error)
        }
    }
}

fn check_command(scratch: bool) -> Result<(), Box<dyn Error>> {
    let parameters = json!({ "n": 10000, "validations": ["v1", "v2", "v3", "v4", "v5", "v6", "v7"], "seeds": [0] });
    let (_recorder, handle) = start_run("check", &parameters, "V-1--V-7", scratch)?;
    let started = Instant::now();
    match run_checks_with_scale(CheckScale::Full) {
        Ok(checks) => {
            let mut log = String::new();
            let mut metrics = Vec::new();
            for check in &checks {
                let line = format!(
                    "{} {} | observed: {} | expected: {}\n",
                    check.id.to_uppercase(),
                    if check.passed { "PASS" } else { "FAIL" },
                    check.observed,
                    check.expected
                );
                print!("{line}");
                log.push_str(&line);
                metrics.push((format!("check.{}", check.id), f64::from(check.passed)));
            }
            let failures = checks.iter().filter(|check| !check.passed).count();
            metrics.push(("checks_failed".to_owned(), failures as f64));
            let borrowed: Vec<_> = metrics.iter().map(|(key, value)| (key.as_str(), *value)).collect();
            handle.with_run(|run| run.log_metrics("run", &borrowed))??;
            write_run_file(&handle, "logs/checks.log", &log)?;
            let elapsed = started.elapsed();
            if failures > 0 {
                let message = format!("{failures} validation checks failed");
                let _ = handle.fail("check", &message);
                return Err(message.into());
            }
            let directory = finish(handle)?;
            println!("Checks passed: {}/{} in {:.3} s", checks.len(), checks.len(), elapsed.as_secs_f64());
            println!("Run directory: {}", directory.display());
            Ok(())
        }
        Err(error) => {
            fail(handle, error.as_ref());
            Err(error)
        }
    }
}
