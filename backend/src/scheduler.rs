//! Tâches planifiées (cron) :
//! - publication automatique des articles validés dont la date est atteinte ;
//! - purge horaire des jetons email expirés.

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{app::AppState, modules::{articles::service, auth::tokens}};

pub async fn start(state: AppState) -> anyhow::Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;

    let publish_state = state.clone();
    scheduler
        .add(Job::new_async(state.config.publish_cron.as_str(), move |_id, _lock| {
            let state = publish_state.clone();
            Box::pin(async move {
                if let Err(err) = service::publish_scheduled(&state).await {
                    tracing::error!(error = ?err, "job de publication en échec");
                }
            })
        })?)
        .await?;

    let purge_state = state.clone();
    scheduler
        .add(Job::new_async("0 17 * * * *", move |_id, _lock| {
            let state = purge_state.clone();
            Box::pin(async move {
                match tokens::purge_expired(&state.db).await {
                    Ok(n) if n > 0 => tracing::info!(n, "jetons expirés supprimés"),
                    Ok(_) => {}
                    Err(err) => tracing::error!(error = ?err, "purge des jetons en échec"),
                }
            })
        })?)
        .await?;

    scheduler.start().await?;
    tracing::info!(cron = %state.config.publish_cron, "planificateur démarré");
    Ok(scheduler)
}
