use crate::{
    builtin::BuiltinOutput,
    job::{Job, JobStatus, recent_jobs_ids},
};

pub fn invoke(mut jobs: Vec<Job>) -> BuiltinOutput {
    if jobs.is_empty() {
        return BuiltinOutput::new(0, "".to_string(), "".to_string());
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

    BuiltinOutput::new(0, output.join("\n"), "".to_string())
}
