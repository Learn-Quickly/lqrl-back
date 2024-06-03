use std::{str::FromStr, sync::Arc};

use chrono::{Local, Utc};
use cron::Schedule;
use lib_core::interactors::cron_job_exercise::CronJobExercise;
use lib_db::store::command_repository_manager::CommandRepositoryManager;

#[tokio::main]
async fn main() {
	let command_repository = Arc::new(CommandRepositoryManager::new().await.unwrap());

    let expression = "0/60 * * * * *";
    let schedule = Schedule::from_str(expression).expect("Failed to parse CRON expression");

    loop {
        let now = Utc::now();
        if let Some(next) = schedule.upcoming(Utc).take(1).next() {
            let cron_ex = CronJobExercise::new(command_repository.clone());

            let until_next = next - now;

            tokio::spawn(async move {
                match cron_ex.complete_overdue_exercises().await {
                    Ok(completed_exercises) => {
                        println!(
                            "Successfully completed {} exercises. Current time: {}",
                            completed_exercises,
                            Local::now().format("%Y-%m-%d %H:%M:%S"),
                        );
                    },
                    Err(_) => {
                        println!(
                            "Failed. Current time: {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S")
                        );
                    },
                }
            }).await.unwrap();  

            tokio::time::sleep(until_next.to_std().unwrap()).await;
        }
    }
}