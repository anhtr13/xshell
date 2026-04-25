use anyhow::Result;

use crate::{
    builtin::BuiltinOutput,
    job::{Job, JobStatus, recent_jobs_ids},
};

pub fn invoke(mut jobs: Vec<Job>) -> Result<BuiltinOutput> {
    if jobs.is_empty() {
        return Ok(BuiltinOutput::default());
    }

    let recent = recent_jobs_ids(&jobs);
    jobs.sort_unstable_by_key(|x| x.number);

    let mut output = Vec::new();

    for job in jobs.iter() {
        let marker = match job.pid {
            id if id == recent.0 => "+",
            id if id == recent.1 => "-",
            _ => " ",
        };
        let space = match job.status {
            JobStatus::Running => "                 ",
            JobStatus::Done => "                    ",
        };
        output.push(format!(
            "[{}]{}  {}{}{}",
            job.number, marker, job.status, space, job.command
        ));
    }

    Ok(BuiltinOutput::new(0, output.join("\n")))
}
